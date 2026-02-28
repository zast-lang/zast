use crate::types::FloatWidth;

#[derive(Debug)]
pub enum AnnotatedType {
    Primitive(String),
    Pointer(Box<AnnotatedType>),
}

impl AnnotatedType {
    pub fn is_int(&self) -> bool {
        match self {
            Self::Primitive(t) => {
                t.starts_with("i") && t[1..].parse::<u16>().map(|n| n >= 1).unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Self::Primitive(t) => {
                t.starts_with("u") && t[1..].parse::<u16>().map(|n| n >= 1).unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::Primitive(t) => {
                t.starts_with("f") && t[1..].parse::<u16>().map(|n| n >= 1).unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn get_float_bitwidth(&self) -> Option<FloatWidth> {
        match self {
            Self::Primitive(t) => {
                if !t.starts_with("f") {
                    return None;
                }
                let bits = t[1..].parse::<u16>().ok()?;
                match bits {
                    16 => Some(FloatWidth::F16),
                    32 => Some(FloatWidth::F32),
                    64 => Some(FloatWidth::F64),
                    128 => Some(FloatWidth::F128),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn get_int_bitwidth(&self) -> Option<u16> {
        match self {
            Self::Primitive(t) => {
                if !t.starts_with("i") {
                    return None;
                }
                let bits = t[1..].parse::<u16>().ok()?;
                if bits == 0 {
                    return None;
                }
                Some(bits)
            }
            _ => None,
        }
    }

    pub fn get_unsigned_bitwidth(&self) -> Option<u16> {
        match self {
            Self::Primitive(t) => {
                if !t.starts_with("u") {
                    return None;
                }
                let bits = t[1..].parse::<u16>().ok()?;
                if bits == 0 {
                    return None;
                }
                Some(bits)
            }
            _ => None,
        }
    }
}
