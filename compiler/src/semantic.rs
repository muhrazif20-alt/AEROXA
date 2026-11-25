use crate::ast::{
    Program,
    Statement,
    Expression,
    BinaryOperator,
    UnaryOperator,
};
use crate::diagnostics::{Diagnostic, SourceLocation};
use crate::symbol_table::{Symbol, SymbolTable};

pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut symbols = SymbolTable::new();

    for statement in &program.statements {
        analyze_statement(statement, &mut symbols, &mut diagnostics);
    }

    diagnostics
}

fn analyze_statement(
    statement: &Statement,
    symbols: &mut SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Link{
            module:_,
            location:_,
        }=>{
            //MODULE WI BE HANDLEDBY THE MODULE RESOLVER
        }
        Statement::Print { value, location } => {
            analyze_expression(value, symbols, diagnostics, location);
        }

        Statement::VariableDeclaration {
            name,
            data_type,
            mutable,
            value,
            location,
            override_double,
        } => {
            if symbols.lookup_current_scope(name).is_some() {
                diagnostics.push(Diagnostic::error(
                    "AX006",
                    &format!("Variable '{}' already declared in this scope.", name),
                    location.line,
                    location.column,
                ));
                return;
            }

            let outer_symbol = symbols.lookup_outer_scope(name).cloned();

            match (&outer_symbol, override_double) {
                (Some(_), false) => {
                    diagnostics.push(Diagnostic::error(
                        "AX006",
                        &format!(
                            "Variable '{}' already exists in an outer scope. Use '@overrideDouble' to shadow it.",
                            name
                        ),
                        location.line,
                        location.column,
                    ));
                    return;
                }

                (None, true) => {
                    diagnostics.push(Diagnostic::error(
                        "AX009",
                        &format!(
                            "'@overrideDouble' requires an existing variable named '{}'.",
                            name
                        ),
                        location.line,
                        location.column,
                    ));
                    return;
                }

                (Some(previous), true) => {
                    if previous.data_type != *data_type {
                        diagnostics.push(Diagnostic::error(
                            "AX010",
                            &format!(
                                "Cannot override '{}': expected the same type '{}', found '{}'.",
                                name,
                                previous.data_type,
                                data_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return;
                    }
                }

                (None, false) => {}
            }

            let value_type = analyze_expression(
                value,
                symbols,
                diagnostics,
                location,
            );

            if let Some(value_type) = value_type {
                if !is_type_compatible(data_type, &value_type) {
                    diagnostics.push(Diagnostic::error(
                        "AX005",
                        &format!(
                            "Type mismatch in variable '{}': expected '{}', found '{}'.",
                            name,
                            data_type,
                            value_type
                        ),
                        location.line,
                        location.column,
                    ));
                    return;
                }
            } else {
                return;
            }

            symbols.declare(Symbol {
                name: name.clone(),
                data_type: data_type.clone(),
                mutable: *mutable,
            });
        }

        Statement::Assignment {
            name,
            value,
            location,
        } => {
            let symbol = match symbols.lookup(name) {
                Some(symbol) => symbol.clone(),

                None => {
                    diagnostics.push(Diagnostic::error(
                        "AX007",
                        &format!("Variable '{}' is not declared.", name),
                        location.line,
                        location.column,
                    ));
                    return;
                }
            };

            if !symbol.mutable {
                diagnostics.push(Diagnostic::error(
                    "AX008",
                    &format!(
                        "Cannot assign to immutable variable '{}'.",
                        name
                    ),
                    location.line,
                    location.column,
                ));
                return;
            }

            let value_type = analyze_expression(
                value,
                symbols,
                diagnostics,
                location,
            );

            if let Some(value_type) = value_type {
                if !is_type_compatible(&symbol.data_type, &value_type) {
                    diagnostics.push(Diagnostic::error(
                        "AX005",
                        &format!(
                            "Type mismatch in assignment to '{}': expected '{}', found '{}'.",
                            name,
                            symbol.data_type,
                            value_type
                        ),
                        location.line,
                        location.column,
                    ));
                }
            }
        }

        Statement::Block { statements, .. } => {
            symbols.enter_scope();

            for statement in statements {
                analyze_statement(statement, symbols, diagnostics);
            }

            symbols.exit_scope();
        }

        Statement::Conditional {
            branches,
            otherwise,
            ..
        } => {
            for branch in branches {
                let condition_type = analyze_expression(
                    &branch.condition,
                    symbols,
                    diagnostics,
                    &branch.location,
                );

                if let Some(condition_type) = condition_type {
                    if condition_type != "Bool" {
                        diagnostics.push(Diagnostic::error(
                            "AX013",
                            &format!(
                                "Condition must be Bool, found '{}'.",
                                condition_type
                            ),
                            branch.location.line,
                            branch.location.column,
                        ));
                    }
                }

                symbols.enter_scope();

                for statement in &branch.statements {
                    analyze_statement(statement, symbols, diagnostics);
                }

                symbols.exit_scope();
            }

            if let Some(statements) = otherwise {
                symbols.enter_scope();

                for statement in statements {
                    analyze_statement(statement, symbols, diagnostics);
                }

                symbols.exit_scope();
            }
        }

        Statement::Loop {
            condition,
            statements,
            location,
        } => {
            let condition_type = analyze_expression(
                condition,
                symbols,
                diagnostics,
                location,
            );

            if let Some(condition_type) = condition_type {
                if condition_type != "Bool" {
                    diagnostics.push(Diagnostic::error(
                        "AX013",
                        &format!(
                            "Loop condition must be Bool, found '{}'.",
                            condition_type
                        ),
                        location.line,
                        location.column,
                    ));
                }
            }

            symbols.enter_scope();

            for statement in statements {
                analyze_statement(statement, symbols, diagnostics);
            }

            symbols.exit_scope();
        }
    }
}

fn analyze_expression(
    expression: &Expression,
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
    location: &SourceLocation,
) -> Option<String> {
    match expression {
        Expression::Number(number) => {
            if number.contains('.') {
                Some("Float64".to_string())
            } else {
                Some("Int32".to_string())
            }
        }

        Expression::Boolean(_) => Some("Bool".to_string()),

        Expression::StringLiteral(_) => Some("String".to_string()),

        Expression::Variable(name) => {
            match symbols.lookup(name) {
                Some(symbol) => Some(symbol.data_type.clone()),

                None => {
                    diagnostics.push(Diagnostic::error(
                        "AX007",
                        &format!("Variable '{}' is not declared.", name),
                        location.line,
                        location.column,
                    ));
                    None
                }
            }
        }

        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left_type = analyze_expression(
                left,
                symbols,
                diagnostics,
                location,
            )?;

            let right_type = analyze_expression(
                right,
                symbols,
                diagnostics,
                location,
            )?;

            match operator {
                BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo => {
                    if left_type != right_type {
                        diagnostics.push(Diagnostic::error(
                            "AX031",
                            &format!(
                                "Arithmetic type mismatch: '{}' and '{}'.",
                                left_type,
                                right_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    if !is_numeric_type(&left_type) {
                        diagnostics.push(Diagnostic::error(
                            "AX032",
                            &format!(
                                "Arithmetic operators require numeric operands, found '{}'.",
                                left_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    if *operator == BinaryOperator::Modulo
                        && !is_integer_type(&left_type)
                    {
                        diagnostics.push(Diagnostic::error(
                            "AX033",
                            &format!(
                                "Modulo requires integer operands, found '{}'.",
                                left_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    Some(left_type)
                }

                BinaryOperator::And | BinaryOperator::Or => {
                    if left_type != "Bool" || right_type != "Bool" {
                        diagnostics.push(Diagnostic::error(
                            "AX018",
                            &format!(
                                "Logical operators require Bool operands, found '{}' and '{}'.",
                                left_type,
                                right_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    Some("Bool".to_string())
                }

                BinaryOperator::Equal | BinaryOperator::NotEqual => {
                    if left_type != right_type {
                        diagnostics.push(Diagnostic::error(
                            "AX014",
                            &format!(
                                "Comparison type mismatch: '{}' and '{}'.",
                                left_type,
                                right_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    Some("Bool".to_string())
                }

                BinaryOperator::Greater
                | BinaryOperator::Less
                | BinaryOperator::GreaterEqual
                | BinaryOperator::LessEqual => {
                    if left_type != right_type {
                        diagnostics.push(Diagnostic::error(
                            "AX014",
                            &format!(
                                "Comparison type mismatch: '{}' and '{}'.",
                                left_type,
                                right_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    if is_numeric_type(&left_type) {
                        Some("Bool".to_string())
                    } else {
                        diagnostics.push(Diagnostic::error(
                            "AX015",
                            &format!(
                                "Operator requires numeric operands, found '{}'.",
                                left_type
                            ),
                            location.line,
                            location.column,
                        ));
                        None
                    }
                }
            }
        }

        Expression::Unary {
            operator,
            operand,
        } => {
            let operand_type = analyze_expression(
                operand,
                symbols,
                diagnostics,
                location,
            )?;

            match operator {
                UnaryOperator::Not => {
                    if operand_type != "Bool" {
                        diagnostics.push(Diagnostic::error(
                            "AX019",
                            &format!(
                                "Logical NOT requires a Bool operand, found '{}'.",
                                operand_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    Some("Bool".to_string())
                }

                UnaryOperator::Negate => {
                    if !is_numeric_type(&operand_type) {
                        diagnostics.push(Diagnostic::error(
                            "AX034",
                            &format!(
                                "Unary minus requires a numeric operand, found '{}'.",
                                operand_type
                            ),
                            location.line,
                            location.column,
                        ));
                        return None;
                    }

                    Some(operand_type)
                }
            }
        }
    }
}

fn is_type_compatible(expected: &str, actual: &str) -> bool {
    expected == actual
}

fn is_numeric_type(data_type: &str) -> bool {
    matches!(
        data_type,
        "Int8" | "Int16" | "Int32" | "Int64"
        | "Float32" | "Float64"
    )
}

fn is_integer_type(data_type: &str) -> bool {
    matches!(
        data_type,
        "Int8" | "Int16" | "Int32" | "Int64"
    )
}