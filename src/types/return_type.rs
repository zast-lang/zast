use crate::types::annotated_type::AnnotatedType;

#[derive(Debug, Clone)]
pub enum ReturnType {
    Void,
    Type(AnnotatedType),
}
