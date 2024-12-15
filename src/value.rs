use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ValueType {
    Bool(bool),
    Nil,
    Number(f64),
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Bool(b) => write!(f,"{}", b),
            ValueType::Nil => write!(f,"nil"),
            ValueType::Number(n) => write!(f, "{}", n),
        }
    }
}

impl ValueType {
    pub fn is_falsey(&self) -> bool {
        match self {
            ValueType::Nil => true,
            ValueType::Bool(b) => !b,
            _ => false
        }
    }
}