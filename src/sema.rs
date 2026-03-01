use std::mem;

use crate::{
    ast::{Statement, Stmt, ZastProgram},
    error_handler::{ZastErrorCollector, zast_errors::ZastError},
    lexer::tokens::Span,
    sema::{symbol_type_table::ZastSymbolTypeTable, type_map::ZastTypeMap},
    types::{ValueType, return_type},
};

pub mod symbol_type_table;
pub mod type_map;

#[derive(Debug)]
pub struct ZastSemanticAnalyzer {
    pub(crate) errors: ZastErrorCollector,
    pub(crate) type_map: ZastTypeMap,
    pub(crate) symbol_type_table: ZastSymbolTypeTable,
}

impl ZastSemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            errors: ZastErrorCollector::new(),
            type_map: ZastTypeMap::new(),
            symbol_type_table: ZastSymbolTypeTable::new(),
        }
    }

    pub fn analyze(&mut self, program: ZastProgram) -> Result<(), ZastErrorCollector> {
        for stmt in &program.body {
            let _ = self.analyze_stmt(stmt);
        }

        if self.errors.has_errors() {
            Err(mem::take(&mut self.errors))
        } else {
            Ok(())
        }
    }

    fn analyze_stmt(&mut self, stmt: &Statement) -> Option<()> {
        match &stmt.node {
            Stmt::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let mut params = Vec::new();

                for param in parameters {
                    params.push(ValueType::from_annotated_type(param.annotated_type.clone()));
                }

                self.declare_function_type(
                    name.clone(),
                    params,
                    ValueType::from_return_type(return_type.clone()),
                    stmt.span,
                );

                self.enter_scope();
                for param in parameters {
                    self.declare_ident_type_mapping(
                        param.name.clone(),
                        ValueType::from_annotated_type(param.annotated_type.clone()),
                        param.span,
                    );
                }

                self.analyze_stmt(body.as_ref())?;
                self.exit_scope();

                Some(())
            }

            Stmt::BlockStatement { statements } => {
                for stmt in statements {
                    self.analyze_stmt(stmt.as_ref())?;
                }

                Some(())
            }
            e => todo!("{:#?}", e),
        }
    }

    fn declare_ident_type_mapping(
        &mut self,
        identifier: String,
        value_type: ValueType,
        span: Span,
    ) -> Option<()> {
        match self
            .symbol_type_table
            .declare_ident_type(identifier, value_type, span)
        {
            Ok(()) => Some(()),
            Err(zast_err) => {
                self.throw_error(zast_err);
                None
            }
        }
    }

    fn declare_function_type(
        &mut self,
        identifier: String,
        params: Vec<ValueType>,
        return_type: ValueType,
        span: Span,
    ) -> Option<()> {
        match self
            .symbol_type_table
            .declare_function_type(identifier, params, return_type, span)
        {
            Ok(()) => Some(()),
            Err(zast_err) => {
                self.throw_error(zast_err);
                None
            }
        }
    }

    fn enter_scope(&mut self) {
        self.symbol_type_table.enter_scope();
    }

    fn exit_scope(&mut self) {
        self.symbol_type_table.exit_scope();
    }

    fn throw_error(&mut self, zast_error: ZastError) {
        self.errors.add_error(zast_error);
    }
}
