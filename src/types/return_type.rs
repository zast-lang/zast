use crate::types::annotated_type::AnnotatedType;

#[derive(Debug)]
pub enum ReturnType {
    Void,
    Type(AnnotatedType),
}
