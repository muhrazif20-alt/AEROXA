use std::collections::HashMap;

use crate::ir::{
    Instruction,
    IrBinaryOperator,
    IrModule,
    IrType,
    IrUnaryOperator,
    Terminator,
    ValueId,
};

fn escape_llvm_string(value: &str) -> String {
    let mut escaped = String::new();

    for byte in value.as_bytes() {
        if (0x20..=0x7E).contains(byte)
            && *byte != b'"'
            && *byte != b'\\'
        {
            escaped.push(*byte as char);
        } else {
            escaped.push_str(&format!("\\{:02X}", byte));
        }
    }

    escaped
}

fn llvm_type(data_type: &IrType) -> Result<&'static str, String> {
    match data_type {
        IrType::Int32 => Ok("i32"),
        IrType::Bool => Ok("i1"),
        IrType::String => Ok("ptr"),
        _ => Err("Unsupported LLVM data type.".to_string()),
    }
}

pub fn generate_llvm_ir(module: &IrModule) -> Result<String, String> {
    let mut output = String::new();

    output.push_str("; AEROXA LLVM Backend\n");
    output.push_str("source_filename = \"aeroxa\"\n\n");

    output.push_str(
    "@.fmt.int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\", align 1\n",
);
output.push_str(
    "@.fmt.str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1\n",
);
output.push_str(
    "@.fmt.int.no_newline = private unnamed_addr constant [3 x i8] c\"%d\\00\", align 1\n",
);
output.push_str(
    "@.fmt.str.no_newline = private unnamed_addr constant [3 x i8] c\"%s\\00\", align 1\n\n",
);

    output.push_str("declare i32 @printf(ptr, ...)\n");
    output.push_str("declare ptr @aeroxa_input_string(ptr)\n");
    output.push_str("declare i32 @aeroxa_input_i32(ptr)\n\n");

    let mut string_constants: HashMap<ValueId, String> = HashMap::new();
    let mut input_prompts: HashMap<ValueId, String> = HashMap::new();

    // Collect global strings and input prompts.
    for function in &module.functions {
        for block in &function.blocks {
            for instruction in &block.instructions {
                match instruction {
                    Instruction::ConstantString { result, value } => {
                        string_constants.insert(*result, value.clone());
                    }

                    Instruction::Input {
                        result,
                        prompt,
                        ..
                    } => {
                        input_prompts.insert(*result, prompt.clone());
                    }

                    _ => {}
                }
            }
        }
    }

    // Emit string constants.
    for (value_id, value) in &string_constants {
        let escaped = escape_llvm_string(value);
        let length = value.as_bytes().len() + 1;

        output.push_str(&format!(
            "@.str{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n",
            value_id.0,
            length,
            escaped
        ));
    }

    // Emit input prompts at module scope.
    for (value_id, prompt) in &input_prompts {
        let escaped = escape_llvm_string(prompt);
        let length = prompt.as_bytes().len() + 1;

        output.push_str(&format!(
            "@.prompt{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n",
            value_id.0,
            length,
            escaped
        ));
    }

    if !string_constants.is_empty() || !input_prompts.is_empty() {
        output.push('\n');
    }

    for function in &module.functions {
        if function.name != "main" {
            return Err(
                "Only the main function is supported currently.".to_string(),
            );
        }

        if function.blocks.is_empty() {
            return Err("Function has no basic blocks.".to_string());
        }

        output.push_str("define i32 @main() {\n");

        let mut slots: HashMap<String, (String, IrType)> = HashMap::new();
        let mut next_slot = 0u32;

        // Allocate variable storage.
        for block in &function.blocks {
            for instruction in &block.instructions {
                if let Instruction::Declare {
                    name,
                    data_type,
                    ..
                } = instruction
                {
                    if !slots.contains_key(name) {
                        let slot = format!("%slot{}", next_slot);
                        next_slot += 1;

                        match data_type {
                            IrType::Int32 | IrType::String => {}
                            _ => {
                                return Err(
                                    "Backend supports Int32 and String variables only."
                                        .to_string(),
                                );
                            }
                        }

                        slots.insert(
                            name.clone(),
                            (slot, data_type.clone()),
                        );
                    }
                }
            }
        }

        output.push_str("entry:\n");

        for (slot, data_type) in slots.values() {
            let (llvm_data_type, alignment) = match data_type {
                IrType::Int32 => ("i32", 4),
                IrType::String => ("ptr", 8),
                _ => {
                    return Err(
                        "Unsupported variable storage type.".to_string(),
                    );
                }
            };

            output.push_str(&format!(
                "  {} = alloca {}, align {}\n",
                slot,
                llvm_data_type,
                alignment
            ));
        }

        let mut value_types: HashMap<ValueId, IrType> = HashMap::new();

        for (block_index, block) in function.blocks.iter().enumerate() {
            if block_index != 0 {
                output.push_str(&format!("block{}:\n", block.id.0));
            }

            for instruction in &block.instructions {
                match instruction {
                    Instruction::ConstantInt {
                        result,
                        value,
                        data_type,
                    } => {
                        if *data_type != IrType::Int32 {
                            return Err(
                                "Only Int32 constants are supported.".to_string(),
                            );
                        }

                        output.push_str(&format!(
                            "  %v{} = add i32 0, {}\n",
                            result.0,
                            value
                        ));

                        value_types.insert(*result, IrType::Int32);
                    }

                    Instruction::ConstantBool { result, value } => {
                        let llvm_value = if *value { "true" } else { "false" };

                        output.push_str(&format!(
                            "  %v{} = add i1 0, {}\n",
                            result.0,
                            llvm_value
                        ));

                        value_types.insert(*result, IrType::Bool);
                    }

                    Instruction::ConstantString { result, value } => {
                        let length = value.as_bytes().len() + 1;

                        output.push_str(&format!(
                            "  %v{} = getelementptr inbounds [{} x i8], ptr @.str{}, i64 0, i64 0\n",
                            result.0,
                            length,
                            result.0
                        ));

                        value_types.insert(*result, IrType::String);
                    }

                    Instruction::ConstantFloat { .. } => {
                        return Err(
                            "Float constants are not supported yet.".to_string(),
                        );
                    }

                    // Runtime input.
                    Instruction::Input {
                        result,
                        data_type,
                        ..
                    } => {
                        let prompt_global = format!("@.prompt{}", result.0);

                        match data_type {
                            IrType::String => {
                                output.push_str(&format!(
                                    "  %v{} = call ptr @aeroxa_input_string(ptr {})\n",
                                    result.0,
                                    prompt_global
                                ));
                            }

                            IrType::Int32 => {
                                output.push_str(&format!(
                                    "  %v{} = call i32 @aeroxa_input_i32(ptr {})\n",
                                    result.0,
                                    prompt_global
                                ));
                            }

                            _ => {
                                return Err(
                                    "input() currently supports String and Int32 only."
                                        .to_string(),
                                );
                            }
                        }

                        value_types.insert(*result, data_type.clone());
                    }

                    Instruction::Load {
                        result,
                        name,
                        data_type,
                    } => {
                        let (slot, slot_type) =
                            slots.get(name).ok_or_else(|| {
                                format!("Unknown variable '{}'", name)
                            })?;

                        if slot_type != data_type {
                            return Err(
                                "Variable load type mismatch.".to_string(),
                            );
                        }

                        let llvm_data_type = llvm_type(data_type)?;

                        let alignment = match data_type {
                            IrType::Int32 => 4,
                            IrType::String => 8,
                            _ => 1,
                        };

                        output.push_str(&format!(
                            "  %v{} = load {}, ptr {}, align {}\n",
                            result.0,
                            llvm_data_type,
                            slot,
                            alignment
                        ));

                        value_types.insert(*result, data_type.clone());
                    }

                    Instruction::Binary {
                        result,
                        left,
                        operator,
                        right,
                        data_type,
                    } => {
                        let left_type = value_types
                            .get(left)
                            .ok_or_else(|| "Unknown left operand.".to_string())?;

                        let right_type = value_types
                            .get(right)
                            .ok_or_else(|| "Unknown right operand.".to_string())?;

                        if left_type != right_type {
                            return Err(
                                "Binary operand type mismatch.".to_string(),
                            );
                        }

                        let left_value = format!("%v{}", left.0);
                        let right_value = format!("%v{}", right.0);

                        let instruction_text = match operator {
                            IrBinaryOperator::Add
                            | IrBinaryOperator::Subtract
                            | IrBinaryOperator::Multiply
                            | IrBinaryOperator::Divide
                            | IrBinaryOperator::Modulo => {
                                if left_type != &IrType::Int32 {
                                    return Err(
                                        "Arithmetic currently supports Int32 only."
                                            .to_string(),
                                    );
                                }

                                match operator {
                                    IrBinaryOperator::Add => {
                                        format!("add i32 {}, {}", left_value, right_value)
                                    }
                                    IrBinaryOperator::Subtract => {
                                        format!("sub i32 {}, {}", left_value, right_value)
                                    }
                                    IrBinaryOperator::Multiply => {
                                        format!("mul i32 {}, {}", left_value, right_value)
                                    }
                                    IrBinaryOperator::Divide => {
                                        format!("sdiv i32 {}, {}", left_value, right_value)
                                    }
                                    IrBinaryOperator::Modulo => {
                                        format!("srem i32 {}, {}", left_value, right_value)
                                    }
                                    _ => unreachable!(),
                                }
                            }

                            IrBinaryOperator::Equal
                            | IrBinaryOperator::NotEqual
                            | IrBinaryOperator::Greater
                            | IrBinaryOperator::Less
                            | IrBinaryOperator::GreaterEqual
                            | IrBinaryOperator::LessEqual => {
                                if left_type != &IrType::Int32 {
                                    return Err(
                                        "Comparisons currently support Int32 only."
                                            .to_string(),
                                    );
                                }

                                let predicate = match operator {
                                    IrBinaryOperator::Equal => "eq",
                                    IrBinaryOperator::NotEqual => "ne",
                                    IrBinaryOperator::Greater => "sgt",
                                    IrBinaryOperator::Less => "slt",
                                    IrBinaryOperator::GreaterEqual => "sge",
                                    IrBinaryOperator::LessEqual => "sle",
                                    _ => unreachable!(),
                                };

                                format!(
                                    "icmp {} i32 {}, {}",
                                    predicate,
                                    left_value,
                                    right_value
                                )
                            }

                            IrBinaryOperator::And | IrBinaryOperator::Or => {
                                if left_type != &IrType::Bool {
                                    return Err(
                                        "Logical operators require Bool operands."
                                            .to_string(),
                                    );
                                }

                                let operation = match operator {
                                    IrBinaryOperator::And => "and",
                                    IrBinaryOperator::Or => "or",
                                    _ => unreachable!(),
                                };

                                format!(
                                    "{} i1 {}, {}",
                                    operation,
                                    left_value,
                                    right_value
                                )
                            }
                        };

                        output.push_str(&format!(
                            "  %v{} = {}\n",
                            result.0,
                            instruction_text
                        ));

                        value_types.insert(*result, data_type.clone());
                    }

                    Instruction::Declare {
                        name,
                        value,
                        data_type,
                        ..
                    } => {
                        let (slot, slot_type) =
                            slots.get(name).ok_or_else(|| {
                                format!("Unknown variable '{}'", name)
                            })?;

                        let value_type = value_types
                            .get(value)
                            .ok_or_else(|| "Unknown declaration value.".to_string())?;

                        if slot_type != data_type || value_type != data_type {
                            return Err(
                                "Declaration type mismatch.".to_string(),
                            );
                        }

                        let llvm_data_type = llvm_type(data_type)?;

                        let alignment = match data_type {
                            IrType::Int32 => 4,
                            IrType::String => 8,
                            _ => 1,
                        };

                        output.push_str(&format!(
                            "  store {} %v{}, ptr {}, align {}\n",
                            llvm_data_type,
                            value.0,
                            slot,
                            alignment
                        ));
                    }

                    Instruction::Store { name, value } => {
                        let (slot, slot_type) =
                            slots.get(name).ok_or_else(|| {
                                format!("Unknown variable '{}'", name)
                            })?;

                        let value_type = value_types
                            .get(value)
                            .ok_or_else(|| "Unknown stored value.".to_string())?;

                        if slot_type != value_type {
                            return Err(
                                "Assignment type mismatch.".to_string(),
                            );
                        }

                        let llvm_data_type = llvm_type(slot_type)?;

                        let alignment = match slot_type {
                            IrType::Int32 => 4,
                            IrType::String => 8,
                            _ => 1,
                        };

                        output.push_str(&format!(
                            "  store {} %v{}, ptr {}, align {}\n",
                            llvm_data_type,
                            value.0,
                            slot,
                            alignment
                        ));
                    }

                    Instruction::Unary {
                        result,
                        operator,
                        operand,
                        data_type,
                    } => {
                        let operand_type = value_types
                            .get(operand)
                            .ok_or_else(|| "Unknown unary operand.".to_string())?;

                        let text = match operator {
                            IrUnaryOperator::Negate => {
                                if operand_type != &IrType::Int32 {
                                    return Err(
                                        "Negation requires Int32.".to_string(),
                                    );
                                }

                                format!("sub i32 0, %v{}", operand.0)
                            }

                            IrUnaryOperator::Not => {
                                if operand_type != &IrType::Bool {
                                    return Err(
                                        "Boolean NOT requires Bool.".to_string(),
                                    );
                                }

                                format!("xor i1 %v{}, true", operand.0)
                            }
                        };

                        output.push_str(&format!(
                            "  %v{} = {}\n",
                            result.0,
                            text
                        ));

                        value_types.insert(*result, data_type.clone());
                    }

                    Instruction::Print {
    value,
    data_type,
    newline,
} => {
    let value_type = value_types
        .get(value)
        .ok_or_else(|| {
            "Unknown value in Print instruction.".to_string()
        })?;

    if value_type != data_type {
        return Err("Print type mismatch.".to_string());
    }

    match data_type {
        IrType::Int32 => {
            let format = if *newline {
                "@.fmt.int"
            } else {
                "@.fmt.int.no_newline"
            };

            output.push_str(&format!(
                "  call i32 (ptr, ...) @printf(ptr {}, i32 %v{})\n",
                format,
                value.0
            ));
        }

        IrType::String => {
            let format = if *newline {
                "@.fmt.str"
            } else {
                "@.fmt.str.no_newline"
            };

            output.push_str(&format!(
                "  call i32 (ptr, ...) @printf(ptr {}, ptr %v{})\n",
                format,
                value.0
            ));
        }

        _ => {
            return Err(
                "Print supports Int32 and String only.".to_string(),
            );
        }
    }
}
                }
            }

            match &block.terminator {
                Some(Terminator::Return(None)) => {
                    output.push_str("  ret i32 0\n");
                }

                Some(Terminator::Jump(target)) => {
                    output.push_str(&format!(
                        "  br label %block{}\n",
                        target.0
                    ));
                }

                Some(Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                }) => {
                    let condition_type = value_types
                        .get(condition)
                        .ok_or_else(|| {
                            "Unknown branch condition.".to_string()
                        })?;

                    if condition_type != &IrType::Bool {
                        return Err(
                            "Branch condition must be Bool.".to_string(),
                        );
                    }

                    output.push_str(&format!(
                        "  br i1 %v{}, label %block{}, label %block{}\n",
                        condition.0,
                        then_block.0,
                        else_block.0
                    ));
                }

                Some(Terminator::Return(Some(_))) => {
                    return Err(
                        "Returning a value is not supported yet.".to_string(),
                    );
                }

                None => {
                    return Err(
                        "Basic block has no terminator.".to_string(),
                    );
                }
            }
        }

        output.push_str("}\n");
    }

    Ok(output)
}