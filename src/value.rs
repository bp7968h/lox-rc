use std::fmt;

use crate::object::Object;
use crate::InterpretError;

#[derive(Debug, Clone)]
pub enum ValueType {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Object),
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Bool(b) => write!(f, "{}", b),
            ValueType::Nil => write!(f, "nil"),
            ValueType::Number(n) => write!(f, "{}", n),
            ValueType::Obj(o) => {
                write!(f, "{}", o)
            }
        }
    }
}

impl ValueType {
    pub fn is_falsey(&self) -> bool {
        match self {
            ValueType::Nil => true,
            ValueType::Bool(b) => !b,
            _ => false,
        }
    }

    pub fn is_obj_type(&self) -> bool {
        matches!(self, ValueType::Obj(_))
    }
}

// This is need to use the ==
impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueType::Bool(a), ValueType::Bool(b)) => a == b,
            (ValueType::Nil, ValueType::Nil) => true,
            (ValueType::Number(a), ValueType::Number(b)) => a == b,
            (ValueType::Obj(a), ValueType::Obj(b)) => a == b,
            _ => false,
        }
    }
}

// This is required to use > and <.
// Also PartialEq is should also be implemented to implement this
impl PartialOrd for ValueType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ValueType::Number(a), ValueType::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

use std::ops::{Add, Div, Mul, Sub};

impl Add for ValueType {
    type Output = Result<Self, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueType::Number(a), ValueType::Number(b)) => Ok(ValueType::Number(a + b)),
            (ValueType::Obj(a), ValueType::Obj(b)) => Ok(ValueType::Obj((a + b)?)),
            _ => Err(InterpretError::RuntimeError),
        }
    }
}

impl Sub for ValueType {
    type Output = Result<Self, InterpretError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueType::Number(a), ValueType::Number(b)) => Ok(ValueType::Number(a - b)),
            _ => Err(InterpretError::RuntimeError),
        }
    }
}

impl Mul for ValueType {
    type Output = Result<Self, InterpretError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueType::Number(a), ValueType::Number(b)) => Ok(ValueType::Number(a * b)),
            _ => Err(InterpretError::RuntimeError),
        }
    }
}

impl Div for ValueType {
    type Output = Result<Self, InterpretError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueType::Number(a), ValueType::Number(b)) => Ok(ValueType::Number(a / b)),
            _ => Err(InterpretError::RuntimeError),
        }
    }
}
