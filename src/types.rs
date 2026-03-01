use crate::types::{annotated_type::AnnotatedType, return_type::ReturnType};

pub mod annotated_type;
pub mod return_type;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FloatWidth {
    F16,
    F32,
    F64,
    F128,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ValueType {
    Integer {
        bits: u16,
        unsigned: bool,
    },
    Float {
        width: FloatWidth,
    },
    Pointer(Box<ValueType>),
    Bool,

    Void, // return type
    Function {
        params: Vec<ValueType>,
        return_type: Box<ValueType>,
    },
}

impl ValueType {
    pub fn from_return_type(return_type: ReturnType) -> Self {
        match return_type {
            ReturnType::Void => Self::Void,
            ReturnType::Type(t) => Self::from_annotated_type(t),
        }
    }

    pub fn from_annotated_type(annotated_type: AnnotatedType) -> Self {
        match annotated_type {
            AnnotatedType::Pointer(a) => {
                let ptr = Self::from_annotated_type(*a);
                Self::Pointer(Box::new(ptr))
            }

            AnnotatedType::Primitive(_) => {
                if annotated_type.is_int() {
                    let width = annotated_type.get_int_bitwidth().unwrap();
                    return Self::Integer {
                        bits: width,
                        unsigned: false,
                    };
                }
                if annotated_type.is_unsigned() {
                    let width = annotated_type.get_unsigned_bitwidth().unwrap();
                    return Self::Integer {
                        bits: width,
                        unsigned: true,
                    };
                }
                if annotated_type.is_float() {
                    let width = annotated_type.get_float_bitwidth().unwrap();
                    return Self::Float { width };
                }
                if annotated_type.is_bool() {
                    return Self::Bool;
                }

                unreachable!()
            }
        }
    }
}
