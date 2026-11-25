#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    String,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrBinaryOperator {
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
pub enum IrUnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    ConstantInt {
        result: ValueId,
        value: i64,
        data_type: IrType,
    },

    ConstantFloat {
        result: ValueId,
        value: f64,
        data_type: IrType,
    },

    ConstantBool {
        result: ValueId,
        value: bool,
    },

    ConstantString {
        result: ValueId,
        value: String,
    },

    Binary {
        result: ValueId,
        left: ValueId,
        operator: IrBinaryOperator,
        right: ValueId,
        data_type: IrType,
    },

    Unary {
        result: ValueId,
        operator: IrUnaryOperator,
        operand: ValueId,
        data_type: IrType,
    },

    Declare {
        name: String,
        data_type: IrType,
        mutable: bool,
        value: ValueId,
    },

    Load {
        result: ValueId,
        name: String,
        data_type: IrType,
    },

    Store {
        name: String,
        value: ValueId,
    },
    Print{
        value:ValueId,
        data_type:IrType,
    },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<ValueId>),
    Jump(BlockId),
    Branch {
        condition: ValueId,
        then_block: BlockId,
        else_block: BlockId,
    },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub return_type: IrType,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub name: String,
    pub functions: Vec<IrFunction>,
}