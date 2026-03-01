use crate::{
    ast::{Stmt, ZastProgram},
    types::{ValueType, return_type::ReturnType},
    zast_ir::ir_instructions::{ZastIRInstruction, ZastIRProgram},
};

pub mod ir_instructions;
pub mod ir_values;

pub struct ZastIREmitter;

impl ZastIREmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, program: &ZastProgram) -> ZastIRProgram {
        let mut instructions = Vec::new();

        for stmt in &program.body {
            if let Some(instr) = self.emit_statement(&stmt.node) {
                instructions.push(instr);
            }
        }

        ZastIRProgram { instructions }
    }

    fn emit_statement(&self, stmt: &Stmt) -> Option<ZastIRInstruction> {
        match stmt {
            Stmt::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let params = parameters
                    .iter()
                    .map(|p| {
                        (
                            p.name.clone(),
                            ValueType::from_annotated_type(p.annotated_type.clone()),
                        )
                    })
                    .collect();

                let ret_ty = match return_type {
                    ReturnType::Void => ValueType::Void,
                    ReturnType::Type(t) => ValueType::from_annotated_type(t.clone()),
                };

                Some(ZastIRInstruction::FunctionDecl {
                    name: name.clone(),
                    params,
                    return_type: ret_ty,
                    body: vec![], // empty for now
                })
            }
            _ => None,
        }
    }
}
