pub mod annotated_type;
pub mod return_type;

#[derive(Debug)]
pub enum FloatWidth {
    F16,
    F32,
    F64,
    F128,
}

#[derive(Debug)]
pub enum ValueType {
    Integer { bits: u16, unsigned: bool },
    Float { width: FloatWidth },
    Pointer(Box<ValueType>),
    Bool,
    String,
}
