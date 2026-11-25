use std::collections::HashMap;
use crate::ast::{
    Program,
    Statement,
    Expression,
};
use crate::diagnostics::Diagnostic;

use super::state::OwnershipInfo;
use super::move_checker::MoveChecker;
use super::borrow_checker::BorrowChecker;
use super::lifetime::{
    LifetimeChecker,
    LifetimeInfo,
};
use super::drop_checker::DropChecker;

pub struct OwnershipAnalyzer {
    scopes: Vec<HashMap<String, OwnershipInfo>>,
    diagnostics: Vec<Diagnostic>,
}

impl OwnershipAnalyzer {
    pub fn analyze_program(
    &mut self,
    program: &Program,
) {
    for statement in &program.statements {
        self.analyze_statement(statement);
    }
}

fn analyze_statement(
    &mut self,
    statement: &Statement,
) {
    match statement {
        Statement::Link{
            module:_,
            location:_,
        }=>{
            //No ownership operation is required for a  module declaration
        }
        Statement::VariableDeclaration {
            name,
            data_type,
            value,
            location: _,
            ..
        } => {
            self.analyze_expression(value);

            self.declare(
                name.clone(),
                data_type.clone(),
            );
        }

        Statement::Assignment {
            name,
            value,
            location,
        } => {
            self.analyze_expression(value);

            self.check_usage(
                name,
                location.line,
                location.column,
            );
        }
        Statement::Print { values, .. } => {
    for value in values {
        self.analyze_expression(value);
    }
}
        Statement::Block {
            statements,
            ..
        } => {
            self.enter_scope();

            for statement in statements {
                self.analyze_statement(statement);
            }

            self.exit_scope();
        }

        Statement::Conditional {
            branches,
            otherwise,
            ..
        } => {
            for branch in branches {
                self.analyze_expression(&branch.condition);

                self.enter_scope();

                for statement in &branch.statements {
                    self.analyze_statement(statement);
                }

                self.exit_scope();
            }

            if let Some(statements) = otherwise {
                self.enter_scope();

                for statement in statements {
                    self.analyze_statement(statement);
                }

                self.exit_scope();
            }
        }
        Statement::Loop {
            condition,
            statements,
            ..
        } => {
            self.analyze_expression(condition);

            self.enter_scope();

            for statement in statements {
                self.analyze_statement(statement);
            }

            self.exit_scope();
        }
    }
}

fn analyze_expression(
    &mut self,
    expression: &Expression,
) {
    match expression {
        Expression::Variable(name) => {
            self.check_usage(name, 0, 0);
        }

        Expression::Binary {
            left,
            right,
            ..
        } => {
            self.analyze_expression(left);
            self.analyze_expression(right);
        }

        Expression::Unary {
            operand,
            ..
        } => {
            self.analyze_expression(operand);
        }

        Expression::Number(_)
        | Expression::StringLiteral(_)
        | Expression::Boolean(_) => {}
        | Expression::Input {..}=> {}
    }
}
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            diagnostics: Vec::new(),
        }
    }

    pub fn declare(
        &mut self,
        name: String,
        data_type: String,
    ) {
        let info = OwnershipInfo::new(
            name.clone(),
            data_type,
        );

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    fn get(
        &self,
        name: &str,
    ) -> Option<&OwnershipInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    fn get_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut OwnershipInfo> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }

    pub fn move_value(
        &mut self,
        name: &str,
        line: usize,
        column: usize,
    ) {
        if let Some(info) = self.get_mut(name) {
            if let Err(error) =
                MoveChecker::move_value(info, line, column)
            {
                self.diagnostics.push(error);
            }
        }
    }

    pub fn check_usage(
        &mut self,
        name: &str,
        line: usize,
        column: usize,
    ) {
        if let Some(info) = self.get(name) {
            if let Err(error) =
                MoveChecker::check_usage(info, line, column)
            {
                self.diagnostics.push(error);
            }
        }
    }

    pub fn borrow(
        &mut self,
        name: &str,
        line: usize,
        column: usize,
    ) {
        if let Some(info) = self.get_mut(name) {
            if let Err(error) =
                BorrowChecker::borrow(info, line, column)
            {
                self.diagnostics.push(error);
            }
        }
    }

    pub fn borrow_mut(
        &mut self,
        name: &str,
        line: usize,
        column: usize,
    ) {
        if let Some(info) = self.get_mut(name) {
            if let Err(error) =
                BorrowChecker::borrow_mut(info, line, column)
            {
                self.diagnostics.push(error);
            }
        }
    }

    pub fn release_borrow(
        &mut self,
        name: &str,
    ) {
        if let Some(info) = self.get_mut(name) {
            BorrowChecker::release(info);
        }
    }

    pub fn check_lifetime(
        &mut self,
        lifetime: &LifetimeInfo,
        line: usize,
        column: usize,
    ) {
        if let Err(error) =
            LifetimeChecker::check(lifetime, line, column)
        {
            self.diagnostics.push(error);
        }
    }

    pub fn drop_value(
        &mut self,
        name: &str,
        line: usize,
        column: usize,
    ) {
        if let Some(info) = self.get_mut(name) {
            if let Err(error) =
                DropChecker::drop_value(info, line, column)
            {
                self.diagnostics.push(error);
            }
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}