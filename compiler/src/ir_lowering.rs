use std::collections::HashMap;

use crate::ast::{
    BinaryOperator,
    Expression,
    Program,
    Statement,
    UnaryOperator,
};

use crate::ir::{
    BasicBlock,
    BlockId,
    Instruction,
    IrBinaryOperator,
    IrFunction,
    IrModule,
    IrType,
    IrUnaryOperator,
    Terminator,
    ValueId,
};

struct LoweringContext {
    next_value: u32,
    next_block: u32,

    instructions: Vec<Instruction>,
    blocks: Vec<BasicBlock>,

    current_block: BlockId,
    variables: HashMap<String, IrType>,
}

impl LoweringContext {
    fn new() -> Self {
        Self {
            next_value: 0,
            next_block: 1,
            instructions: Vec::new(),
            blocks: Vec::new(),
            current_block: BlockId(0),
            variables: HashMap::new(),
        }
    }

    fn new_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value);
        self.next_value += 1;
        id
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        id
    }

    fn terminate_current(&mut self, terminator: Terminator) {
        let block = BasicBlock {
            id: self.current_block,
            instructions: std::mem::take(&mut self.instructions),
            terminator: Some(terminator),
        };

        self.blocks.push(block);
    }

    // ==============================================
    // STATEMENT LOWERING
    // ==============================================

    fn lower_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), String> {
        match statement {
            Statement::Link {
                module: _,
                location: _,
            } => {
                // Module declarations do not generate runtime instructions.
            }

            Statement::VariableDeclaration {
                name,
                data_type,
                mutable,
                value,
                ..
            } => {
                let declared_type = parse_type(data_type)?;

                let (value_id, value_type) =
                    self.lower_expression_expected(
                        value,
                        Some(declared_type.clone()),
                    )?;

                if declared_type != value_type {
                    return Err(format!(
                        "IR type mismatch for variable '{}'",
                        name
                    ));
                }

                self.instructions.push(
                    Instruction::Declare {
                        name: name.clone(),
                        data_type: declared_type.clone(),
                        mutable: *mutable,
                        value: value_id,
                    },
                );

                self.variables.insert(
                    name.clone(),
                    declared_type,
                );
            }

            Statement::Assignment {
                name,
                value,
                ..
            } => {
                let variable_type = self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Unknown variable '{}' during IR lowering",
                            name
                        )
                    })?;

                let (value_id, value_type) =
                    self.lower_expression(value)?;

                if variable_type != value_type {
                    return Err(format!(
                        "Assignment type mismatch for '{}'",
                        name
                    ));
                }

                self.instructions.push(
                    Instruction::Store {
                        name: name.clone(),
                        value: value_id,
                    },
                );
            }

           Statement::Print { values, .. } => {
    let total = values.len();

    for (index, value) in values.iter().enumerate() {
        let (value_id, value_type) =
            self.lower_expression(value)?;

        self.instructions.push(
            Instruction::Print {
                value: value_id,
                data_type: value_type,
                newline: index == total - 1,
            },
        );
    }
}

            Statement::Block { statements, .. } => {
                for statement in statements {
                    self.lower_statement(statement)?;
                }
            }

            Statement::Conditional {
                branches,
                otherwise,
                ..
            } => {
                self.lower_conditional(branches, otherwise)?;
            }

            Statement::Loop {
                condition,
                statements,
                ..
            } => {
                self.lower_loop(condition, statements)?;
            }
        }

        Ok(())
    }

    // ==============================================
    // LOOP LOWERING
    // ==============================================

    fn lower_loop(
        &mut self,
        condition: &Expression,
        statements: &[Statement],
    ) -> Result<(), String> {
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let exit_block = self.new_block();

        self.terminate_current(
            Terminator::Jump(condition_block),
        );

        self.current_block = condition_block;

        let (condition_id, condition_type) =
            self.lower_expression(condition)?;

        if condition_type != IrType::Bool {
            return Err(
                "Loop condition must have Bool type.".to_string()
            );
        }

        self.terminate_current(
            Terminator::Branch {
                condition: condition_id,
                then_block: body_block,
                else_block: exit_block,
            },
        );

        self.current_block = body_block;

        for statement in statements {
            self.lower_statement(statement)?;
        }

        self.terminate_current(
            Terminator::Jump(condition_block),
        );

        self.current_block = exit_block;

        Ok(())
    }

    // ==============================================
    // CONDITIONAL LOWERING
    // ==============================================

    fn lower_conditional(
        &mut self,
        branches: &[crate::ast::ConditionalBranch],
        otherwise: &Option<Vec<Statement>>,
    ) -> Result<(), String> {
        if branches.is_empty() {
            return Ok(());
        }

        let merge_block = self.new_block();

        for (index, branch) in branches.iter().enumerate() {
            let (condition_id, condition_type) =
                self.lower_expression(&branch.condition)?;

            if condition_type != IrType::Bool {
                return Err(
                    "Conditional expression must have Bool type."
                        .to_string()
                );
            }

            let then_block = self.new_block();

            let has_next_branch =
                index + 1 < branches.len()
                    || otherwise.is_some();

            let else_block = if has_next_branch {
                self.new_block()
            } else {
                merge_block
            };

            self.terminate_current(
                Terminator::Branch {
                    condition: condition_id,
                    then_block,
                    else_block,
                },
            );

            self.current_block = then_block;

            for statement in &branch.statements {
                self.lower_statement(statement)?;
            }

            self.terminate_current(
                Terminator::Jump(merge_block),
            );

            self.current_block = else_block;
        }

        if let Some(statements) = otherwise {
            for statement in statements {
                self.lower_statement(statement)?;
            }

            self.terminate_current(
                Terminator::Jump(merge_block),
            );
        }

        self.current_block = merge_block;

        Ok(())
    }

    // ==============================================
    // EXPRESSION LOWERING
    // ==============================================

    fn lower_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<(ValueId, IrType), String> {
        self.lower_expression_expected(expression, None)
    }

    fn lower_expression_expected(
        &mut self,
        expression: &Expression,
        expected_type: Option<IrType>,
    ) -> Result<(ValueId, IrType), String> {
        match expression {
            Expression::Number(number) => {
                let result = self.new_value();

                if let Ok(value) = number.parse::<i64>() {
                    self.instructions.push(
                        Instruction::ConstantInt {
                            result,
                            value,
                            data_type: IrType::Int32,
                        },
                    );

                    return Ok((result, IrType::Int32));
                }

                if let Ok(value) = number.parse::<f64>() {
                    self.instructions.push(
                        Instruction::ConstantFloat {
                            result,
                            value,
                            data_type: IrType::Float64,
                        },
                    );

                    return Ok((result, IrType::Float64));
                }

                Err(format!(
                    "Invalid numeric literal '{}'",
                    number
                ))
            }

            Expression::StringLiteral(value) => {
                let result = self.new_value();

                self.instructions.push(
                    Instruction::ConstantString {
                        result,
                        value: value.clone(),
                    },
                );

                Ok((result, IrType::String))
            }

            Expression::Input { prompt } => {
                let data_type = expected_type.ok_or_else(|| {
                    "input() must initialize a typed variable."
                        .to_string()
                })?;

                if data_type != IrType::String
                    && data_type != IrType::Int32
                {
                    return Err(
                        "input() supports String and Int32 only."
                            .to_string()
                    );
                }

                let result = self.new_value();

                self.instructions.push(
                    Instruction::Input {
                        result,
                        prompt: prompt.clone(),
                        data_type: data_type.clone(),
                    },
                );

                Ok((result, data_type))
            }

            Expression::Boolean(value) => {
                let result = self.new_value();

                self.instructions.push(
                    Instruction::ConstantBool {
                        result,
                        value: *value,
                    },
                );

                Ok((result, IrType::Bool))
            }

            Expression::Variable(name) => {
                let data_type = self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Unknown variable '{}' during IR lowering",
                            name
                        )
                    })?;

                let result = self.new_value();

                self.instructions.push(
                    Instruction::Load {
                        result,
                        name: name.clone(),
                        data_type: data_type.clone(),
                    },
                );

                Ok((result, data_type))
            }

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let (left_id, left_type) =
                    self.lower_expression(left)?;

                let (right_id, right_type) =
                    self.lower_expression(right)?;

                if left_type != right_type {
                    return Err(
                        "Binary expression has incompatible IR types."
                            .to_string()
                    );
                }

                let ir_operator =
                    map_binary_operator(operator)?;

                let result_type = match operator {
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::Less
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::LessEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => IrType::Bool,

                    _ => left_type.clone(),
                };

                let result = self.new_value();

                self.instructions.push(
                    Instruction::Binary {
                        result,
                        left: left_id,
                        operator: ir_operator,
                        right: right_id,
                        data_type: result_type.clone(),
                    },
                );

                Ok((result, result_type))
            }

            Expression::Unary {
                operator,
                operand,
            } => {
                let (operand_id, operand_type) =
                    self.lower_expression(operand)?;

                let ir_operator = match operator {
                    UnaryOperator::Negate =>
                        IrUnaryOperator::Negate,

                    UnaryOperator::Not =>
                        IrUnaryOperator::Not,
                };

                let result_type = match operator {
                    UnaryOperator::Negate =>
                        operand_type.clone(),

                    UnaryOperator::Not =>
                        IrType::Bool,
                };

                let result = self.new_value();

                self.instructions.push(
                    Instruction::Unary {
                        result,
                        operator: ir_operator,
                        operand: operand_id,
                        data_type: result_type.clone(),
                    },
                );

                Ok((result, result_type))
            }
        }
    }
}

// ==============================================
// TYPE PARSER
// ==============================================

fn parse_type(data_type: &str) -> Result<IrType, String> {
    match data_type {
        "Int8" => Ok(IrType::Int8),
        "Int16" => Ok(IrType::Int16),
        "Int32" => Ok(IrType::Int32),
        "Int64" => Ok(IrType::Int64),
        "Float32" => Ok(IrType::Float32),
        "Float64" => Ok(IrType::Float64),
        "Bool" => Ok(IrType::Bool),
        "String" => Ok(IrType::String),

        _ => Err(format!(
            "Unsupported IR type '{}'",
            data_type
        )),
    }
}

// ==============================================
// BINARY OPERATOR MAPPING
// ==============================================

fn map_binary_operator(
    operator: &BinaryOperator,
) -> Result<IrBinaryOperator, String> {
    match operator {
        BinaryOperator::Add =>
            Ok(IrBinaryOperator::Add),

        BinaryOperator::Subtract =>
            Ok(IrBinaryOperator::Subtract),

        BinaryOperator::Multiply =>
            Ok(IrBinaryOperator::Multiply),

        BinaryOperator::Divide =>
            Ok(IrBinaryOperator::Divide),

        BinaryOperator::Modulo =>
            Ok(IrBinaryOperator::Modulo),

        BinaryOperator::Equal =>
            Ok(IrBinaryOperator::Equal),

        BinaryOperator::NotEqual =>
            Ok(IrBinaryOperator::NotEqual),

        BinaryOperator::Greater =>
            Ok(IrBinaryOperator::Greater),

        BinaryOperator::Less =>
            Ok(IrBinaryOperator::Less),

        BinaryOperator::GreaterEqual =>
            Ok(IrBinaryOperator::GreaterEqual),

        BinaryOperator::LessEqual =>
            Ok(IrBinaryOperator::LessEqual),

        BinaryOperator::And =>
            Ok(IrBinaryOperator::And),

        BinaryOperator::Or =>
            Ok(IrBinaryOperator::Or),
    }
}

// ==============================================
// AEROXA PROGRAM LOWERING
// ==============================================

pub fn lower_program(
    program: &Program,
) -> Result<IrModule, String> {
    let mut context = LoweringContext::new();

    for statement in &program.statements {
        context.lower_statement(statement)?;
    }

    context.terminate_current(
        Terminator::Return(None),
    );

    let function = IrFunction {
        name: "main".to_string(),
        return_type: IrType::Void,
        blocks: context.blocks,
    };

    Ok(IrModule {
        name: "AEROXA_MODULE".to_string(),
        functions: vec![function],
    })
}