pub enum ZastIRValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Reference(String),
    Temporary(usize),
    Null,
}
