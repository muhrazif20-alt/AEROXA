use crate::ast::{
    Program,
    Statement,
    Expression,
    BinaryOperator,
    UnaryOperator,
    ConditionalBranch,
};

use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, LocatedToken};

pub struct Parser {
    tokens: Vec<LocatedToken>,
    position: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<LocatedToken>) -> Self {
        Self {
            tokens,
            position: 0,
            diagnostics: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position].token
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();

        if !matches!(token, Token::EOF) {
            self.position += 1;
        }

        token
    }

    fn report_error(&mut self, message: &str) {
        let location = self.tokens[self.position].location.clone();

        self.diagnostics.push(Diagnostic::error(
            "AX004",
            message,
            location.line,
            location.column,
        ));
    }

    fn expect(&mut self, expected: Token, message: &str) -> bool {
        if *self.current() == expected {
            self.advance();
            true
        } else {
            self.report_error(message);
            false
        }
    }

    // ==========================================
    // PRINT
    // ==========================================

    fn parse_print(&mut self) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        self.advance();

        if !self.expect(
            Token::LeftParen,
            "Expected '(' after print.",
        ) {
            return None;
        }

        let mut values = Vec::new();

        // Parse argument pertama.
        values.push(self.parse_expression()?);

        // Parse argument berikutnya jika ada koma.
        while *self.current() == Token::Comma {
            self.advance();

            values.push(self.parse_expression()?);
        }

        if !self.expect(
            Token::RightParen,
            "Expected ')' after print arguments.",
        ) {
            return None;
        }

        if !self.expect(
            Token::Semicolon,
            "Expected ';' after print.",
        ) {
            return None;
        }

        Some(Statement::Print {
            values,
            location,
        })
    }

    // ==========================================
    // ANNOTATIONS
    // ==========================================

    fn parse_annotation(&mut self) -> Option<Statement> {
        self.advance();

        let annotation = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                self.report_error("Expected annotation name after '@'.");
                return None;
            }
        };

        if annotation != "overrideDouble" {
            self.report_error("Unknown annotation.");
            return None;
        }

        if *self.current() != Token::Let {
            self.report_error(
                "Annotation '@overrideDouble' must precede a variable declaration."
            );
            return None;
        }

        self.parse_variable(true)
    }
// ==========================================
// INPUT
// ==========================================

fn parse_input(&mut self) -> Option<Expression> {
    if !self.expect(
        Token::LeftParen,
        "Expected '(' after input.",
    ) {
        return None;
    }

    let prompt = match self.advance() {
        Token::StringLiteral(value) => value,
        _ => {
            self.report_error(
                "Expected string prompt inside input()."
            );
            return None;
        }
    };

    if !self.expect(
        Token::RightParen,
        "Expected ')' after input prompt.",
    ) {
        return None;
    }

    Some(Expression::Input {
        prompt,
    })
}
    // ==========================================
    // EXPRESSIONS
    // ==========================================

    fn parse_primary(&mut self) -> Option<Expression> {
        match self.advance() {
            Token::Boolean(value) => {
                Some(Expression::Boolean(value))
            }

            Token::Number(number) => {
                Some(Expression::Number(number))
            }

            Token::StringLiteral(value) => {
                Some(Expression::StringLiteral(value))
            }
            Token::Identifier(name) if name == "input"=>{
                self.parse_input()
            }
            Token::Identifier(name) => {
                Some(Expression::Variable(name))
            }

            Token::LeftParen => {
                let expression = self.parse_expression()?;

                if !self.expect(
                    Token::RightParen,
                    "Expected ')' after expression.",
                ) {
                    return None;
                }

                Some(expression)
            }

            _ => {
                self.report_error("Expected expression.");
                None
            }
        }
    }

    // ==========================================
    // UNARY
    // ==========================================

    fn parse_unary(&mut self) -> Option<Expression> {
        if *self.current() == Token::Bang {
            self.advance();

            let operand = self.parse_unary()?;

            return Some(Expression::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            });
        }

        if *self.current() == Token::Minus {
            self.advance();

            let operand = self.parse_unary()?;

            return Some(Expression::Unary {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
            });
        }

        self.parse_primary()
    }

    // ==========================================
    // MULTIPLICATIVE
    // ==========================================

    fn parse_multiplicative(&mut self) -> Option<Expression> {
        let mut expression = self.parse_unary()?;

        loop {
            let operator = match self.current() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Some(expression)
    }

    // ==========================================
    // ADDITIVE
    // ==========================================

    fn parse_additive(&mut self) -> Option<Expression> {
        let mut expression = self.parse_multiplicative()?;

        loop {
            let operator = match self.current() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Some(expression)
    }

    // ==========================================
    // COMPARISON
    // ==========================================

    fn parse_comparison(&mut self) -> Option<Expression> {
        let mut expression = self.parse_additive()?;

        let operator = match self.current() {
            Token::EqualEqual => BinaryOperator::Equal,
            Token::NotEqual => BinaryOperator::NotEqual,
            Token::Greater => BinaryOperator::Greater,
            Token::Less => BinaryOperator::Less,
            Token::GreaterEqual => BinaryOperator::GreaterEqual,
            Token::LessEqual => BinaryOperator::LessEqual,
            _ => return Some(expression),
        };

        self.advance();

        let right = self.parse_additive()?;

        expression = Expression::Binary {
            left: Box::new(expression),
            operator,
            right: Box::new(right),
        };

        Some(expression)
    }

    // ==========================================
    // LOGICAL AND
    // ==========================================

    fn parse_and(&mut self) -> Option<Expression> {
        let mut expression = self.parse_comparison()?;

        while *self.current() == Token::AndAnd {
            self.advance();

            let right = self.parse_comparison()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Some(expression)
    }

    // ==========================================
    // LOGICAL OR
    // ==========================================

    fn parse_or(&mut self) -> Option<Expression> {
        let mut expression = self.parse_and()?;

        while *self.current() == Token::OrOr {
            self.advance();

            let right = self.parse_and()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Some(expression)
    }

    // ==========================================
    // MAIN EXPRESSION
    // ==========================================

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_or()
    }

    // ==========================================
    // VARIABLES
    // ==========================================

    fn parse_variable(
        &mut self,
        override_double: bool,
    ) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        self.advance();

        let mutable = if *self.current() == Token::Mut {
            self.advance();
            true
        } else {
            false
        };

        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                self.report_error("Expected variable name.");
                return None;
            }
        };

        if !self.expect(
            Token::Colon,
            "Expected ':' after variable name.",
        ) {
            return None;
        }

        let data_type = match self.advance() {
            Token::Identifier(data_type) => data_type,
            _ => {
                self.report_error("Expected data type.");
                return None;
            }
        };

        if !self.expect(
            Token::Equal,
            "Expected '=' before value.",
        ) {
            return None;
        }

        let value = self.parse_expression()?;

        if !self.expect(
            Token::Semicolon,
            "Expected ';' after declaration.",
        ) {
            return None;
        }

        Some(Statement::VariableDeclaration {
            name,
            data_type,
            mutable,
            value,
            override_double,
            location,
        })
    }
    // ==========================================
// MODULE LINKING
// ==========================================

fn parse_link(&mut self) -> Option<Statement> {
    let location = self.tokens[self.position].location.clone();

    self.advance(); // Consume "link"

    if !self.expect(
        Token::DoubleColon,
        "Expected '::' after 'link'.",
    ) {
        return None;
    }

    let module = match self.advance() {
        Token::Identifier(name) => name,
        _ => {
            self.report_error("Expected module name after 'link::'.");
            return None;
        }
    };

    if !self.expect(
        Token::Semicolon,
        "Expected ';' after module link.",
    ) {
        return None;
    }

    Some(Statement::Link {
        module,
        location,
    })
}

    // ==========================================
    // ASSIGNMENT
    // ==========================================

    fn parse_assignment(&mut self) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => {
                self.report_error("Expected variable name.");
                return None;
            }
        };

        if !self.expect(
            Token::Equal,
            "Expected '=' in assignment.",
        ) {
            return None;
        }

        let value = self.parse_expression()?;

        if !self.expect(
            Token::Semicolon,
            "Expected ';' after assignment.",
        ) {
            return None;
        }

        Some(Statement::Assignment {
            name,
            value,
            location,
        })
    }

    // ==========================================
    // BLOCKS
    // ==========================================

    fn parse_block_contents(&mut self) -> Option<Vec<Statement>> {
        if !self.expect(
            Token::LeftBrace,
            "Expected '{' to begin block.",
        ) {
            return None;
        }

        let mut statements = Vec::new();

        while !matches!(
            self.current(),
            Token::RightBrace | Token::EOF
        ) {
            let previous_position = self.position;

            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.synchronize();
            }

            if self.position == previous_position
                && *self.current() != Token::EOF
                && *self.current() != Token::RightBrace
            {
                self.advance();
            }
        }

        if !self.expect(
            Token::RightBrace,
            "Expected '}' to close block.",
        ) {
            return None;
        }

        Some(statements)
    }

    fn parse_block(&mut self) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        let statements = self.parse_block_contents()?;

        Some(Statement::Block {
            statements,
            location,
        })
    }

    // ==========================================
    // CONDITIONAL
    // ==========================================

    fn parse_conditional(&mut self) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        let mut branches = Vec::new();

        self.advance();

        let condition = self.parse_expression()?;
        let statements = self.parse_block_contents()?;

        branches.push(ConditionalBranch {
            condition,
            statements,
            location: location.clone(),
        });

        while *self.current() == Token::Alternative {
            let alternative_location =
                self.tokens[self.position].location.clone();

            self.advance();

            let condition = self.parse_expression()?;
            let statements = self.parse_block_contents()?;

            branches.push(ConditionalBranch {
                condition,
                statements,
                location: alternative_location,
            });
        }

        let otherwise = if *self.current() == Token::Else {
            self.advance();
            Some(self.parse_block_contents()?)
        } else {
            None
        };

        Some(Statement::Conditional {
            branches,
            otherwise,
            location,
        })
    }

    // ==========================================
    // LOOP
    // ==========================================

    fn parse_loop(&mut self) -> Option<Statement> {
        let location = self.tokens[self.position].location.clone();

        self.advance();

        let condition = self.parse_expression()?;
        let statements = self.parse_block_contents()?;

        Some(Statement::Loop {
            condition,
            statements,
            location,
        })
    }

    // ==========================================
    // STATEMENTS
    // ==========================================

    fn parse_statement(&mut self) -> Option<Statement> {
    match self.current() {
        Token::Let => self.parse_variable(false),

        Token::At => self.parse_annotation(),

        Token::LeftBrace => self.parse_block(),

        Token::Condition => self.parse_conditional(),

        Token::LoopCheck => self.parse_loop(),

        Token::Identifier(name) if name == "link" => {
            self.parse_link()
        }

        Token::Identifier(name) if name == "print" => {
            self.parse_print()
        }

        Token::Identifier(_) => self.parse_assignment(),

        _ => {
            self.report_error(
                "Unexpected token at statement level."
            );
            None
        }
    }
}

    // ==========================================
    // ERROR RECOVERY
    // ==========================================

    fn synchronize(&mut self) {
        let start_position = self.position;

        while !matches!(
            self.current(),
            Token::EOF | Token::Semicolon | Token::RightBrace
        ) {
            self.advance();
        }

        if *self.current() == Token::Semicolon {
            self.advance();
        }

        if self.position == start_position
            && *self.current() != Token::EOF
            && *self.current() != Token::RightBrace
        {
            self.advance();
        }
    }

    // ==========================================
    // PARSE ENTRY POINT
    // ==========================================

    pub fn parse(mut self) -> (Program, Vec<Diagnostic>) {
        let mut statements = Vec::new();

        while !matches!(self.current(), Token::EOF) {
            let previous_position = self.position;

            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.synchronize();
            }

            if self.position == previous_position
                && *self.current() != Token::EOF
            {
                self.advance();
            }
        }

        (
            Program { statements },
            self.diagnostics,
        )
    }
}