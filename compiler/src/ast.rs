use crate::diagnostics::SourceLocation;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Link {
        module: String,
        location: SourceLocation,
    },

    VariableDeclaration {
        name: String,
        data_type: String,
        mutable: bool,
        value: Expression,
        override_double: bool,
        location: SourceLocation,
    },

    Assignment {
        name: String,
        value: Expression,
        location: SourceLocation,
    },

    Print {
        values: Vec<Expression>,
        location: SourceLocation,
    },

    Block {
        statements: Vec<Statement>,
        location: SourceLocation,
    },

    Conditional {
        branches: Vec<ConditionalBranch>,
        otherwise: Option<Vec<Statement>>,
        location: SourceLocation,
    },

    Loop {
        condition: Expression,
        statements: Vec<Statement>,
        location: SourceLocation,
    },
}

#[derive(Debug, Clone)]
pub struct ConditionalBranch {
    pub condition: Expression,
    pub statements: Vec<Statement>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(String),
    StringLiteral(String),
    Boolean(bool),
    Variable(String),

    Input {
        prompt: String,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}