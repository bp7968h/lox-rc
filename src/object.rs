use std::fmt::Display;
use std::ops::Add;

use crate::chunk::Chunk;
use crate::InterpretError;

#[derive(Debug, Clone)]
pub enum Object {
    ObjFunction(ObjFunction),
    ObjString(ObjString),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::ObjString(os) => write!(f, "{}", os),
            Object::ObjFunction(of) => write!(f, "{}", of),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::ObjString(a), Object::ObjString(b)) => a.0 == b.0,
            _ => false,
        }
    }
}

impl Add for Object {
    type Output = Result<Self, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::ObjString(a), Object::ObjString(b)) => {
                let mut new_str = a.0;
                new_str.push_str(&b.0);

                Ok(Object::ObjString(ObjString(new_str)))
            }
            _ => Err(InterpretError::RuntimeError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjString(String);

impl ObjString {
    pub fn new(source: String) -> Self {
        ObjString(source)
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ObjFunction {
    _arity: u32,
    _chunk: Chunk,
    name: Option<ObjString>,
}

impl ObjFunction {
    pub fn new_function() -> ObjFunction {
        ObjFunction {
            _arity: 0,
            _chunk: Chunk::default(),
            name: None,
        }
    }
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(fn_name) = self.name.as_ref() {
            write!(f, "<fn {}>", fn_name)
        } else {
            write!(f, "<fn no_name>")
        }
    }
}