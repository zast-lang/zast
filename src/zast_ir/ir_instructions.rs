use crate::{types::ValueType, zast_ir::ir_values::ZastIRValue};

pub enum ZastIRInstruction {
    // variable declaration
    Declare {
        name: String,
        val_type: ValueType,
        value: ZastIRValue,
        mutable: bool,
    },

    // assignment
    Assign {
        name: String,
        value: ZastIRValue,
    },

    // binary op — always produces a temporary
    BinaryOp {
        dest: usize, // %0
        op: BinaryOp,
        left: ZastIRValue,
        right: ZastIRValue,
        val_type: ValueType,
    },

    // unary op
    UnaryOp {
        dest: usize,
        op: UnaryOp,
        operand: ZastIRValue,
        val_type: ValueType,
    },

    // function declaration
    FunctionDecl {
        name: String,
        params: Vec<(String, ValueType)>,
        return_type: ValueType,
        body: Vec<ZastIRInstruction>,
    },

    // function call
    Call {
        dest: Option<usize>, // None if return is void
        name: String,
        args: Vec<ZastIRValue>,
    },

    // return
    Return(Option<ZastIRValue>),
}

pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum UnaryOp {
    Negate,
    Deref,
    Address,
}

pub struct ZastIRProgram {
    pub instructions: Vec<ZastIRInstruction>,
}
