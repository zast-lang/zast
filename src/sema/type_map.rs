use std::collections::HashMap;

use crate::types::{ValueType, annotated_type::AnnotatedType};

#[derive(Debug)]
pub struct ZastTypeMap {
    type_map: HashMap<AnnotatedType, ValueType>,
}

impl ZastTypeMap {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, annotated_type: AnnotatedType, value_type: ValueType) {
        self.type_map.insert(annotated_type, value_type);
    }

    pub fn resolve_mapping(&mut self, annotated_type: AnnotatedType) -> Option<&ValueType> {
        self.type_map.get(&annotated_type)
    }
}
