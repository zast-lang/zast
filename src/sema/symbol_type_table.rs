use std::collections::HashMap;

use crate::{error_handler::zast_errors::ZastError, lexer::tokens::Span, types::ValueType};

#[derive(Debug)]
pub struct SymbolType {
    value_type: ValueType,
    span: Span,
}

#[derive(Debug)]
pub struct SymbolTypeScope {
    symbols: HashMap<String, SymbolType>,
    symbol_count: usize,
}

impl SymbolTypeScope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            symbol_count: 0,
        }
    }

    fn declare_function_type(
        &mut self,
        identifier: String,
        params: Vec<ValueType>,
        return_type: ValueType,
        span: Span,
    ) -> Result<(), ZastError> {
        let symbol_type = SymbolType {
            value_type: ValueType::Function {
                params,
                return_type: Box::new(return_type),
            },
            span,
        };

        if let Some(original) = self.symbols.insert(identifier.clone(), symbol_type) {
            return Err(ZastError::FunctionRedeclaration {
                span,
                fn_name: identifier.clone(),
                original_span: original.span,
            });
        }

        self.symbol_count += 1;
        Ok(())
    }

    pub fn declare_ident_type(
        &mut self,
        identifier: String,
        value_type: ValueType,
        span: Span,
    ) -> Result<(), ZastError> {
        let symbol_type = SymbolType { value_type, span };

        if let Some(original) = self.symbols.insert(identifier.clone(), symbol_type) {
            return Err(ZastError::VariableRedeclaration {
                span: span,
                variable_name: identifier,
                original_span: original.span,
            });
        }

        self.symbol_count += 1;
        Ok(())
    }

    pub fn get_ident_type(&mut self, identifier: &str) -> Option<&SymbolType> {
        self.symbols.get(identifier)
    }
}

#[derive(Debug)]
pub struct ZastSymbolTypeTable {
    scopes: Vec<SymbolTypeScope>,
    scope_depth: usize,
}

impl ZastSymbolTypeTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![SymbolTypeScope::new()],
            scope_depth: 0,
        }
    }

    pub fn declare_ident_type(
        &mut self,
        identifier: String,
        value_type: ValueType,
        span: Span,
    ) -> Result<(), ZastError> {
        let scope = self.current_scope();
        scope.declare_ident_type(identifier, value_type, span)
    }

    pub fn declare_function_type(
        &mut self,
        identifier: String,
        params: Vec<ValueType>,
        return_type: ValueType,
        span: Span,
    ) -> Result<(), ZastError> {
        let scope = self.current_scope();
        scope.declare_function_type(identifier, params, return_type, span)
    }

    pub fn resolve_ident_type(&mut self, identifier: &str) -> Option<&SymbolType> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(t) = scope.get_ident_type(identifier) {
                return Some(t);
            }
        }

        None
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(SymbolTypeScope::new());
        self.scope_depth += 1;
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
        self.scope_depth -= 1;
    }

    fn current_scope(&mut self) -> &mut SymbolTypeScope {
        &mut self.scopes[self.scope_depth]
    }
}
