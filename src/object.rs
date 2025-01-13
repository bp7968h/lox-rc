use std::fmt::Display;
use std::ops::Add;

use crate::InterpretError;

#[derive(Debug, Clone)]
pub struct Object {
    obj_type: ObjectType,
}

impl Object {
    pub fn new(obj_type: ObjectType) -> Self {
        Object { obj_type }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.obj_type == other.obj_type
    }
}

impl Add for Object {
    type Output = Result<Self, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        match self.obj_type + rhs.obj_type {
            Ok(new_obj) => Ok(Object::new(new_obj)),
            Err(e) => Err(e),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.obj_type)
    }
}

#[derive(Debug, Clone)]
pub enum ObjectType {
    ObjString(String),
}

impl PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectType::ObjString(a), ObjectType::ObjString(b)) => a == b,
        }
    }
}

impl Add for ObjectType {
    type Output = Result<Self, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectType::ObjString(a), ObjectType::ObjString(b)) => {
                let mut new_str = a;
                new_str.push_str(&b);

                Ok(ObjectType::ObjString(new_str))
            }
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::ObjString(s) => write!(f, "{}", s),
        }
    }
}
