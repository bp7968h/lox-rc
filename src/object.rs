use std::fmt::Display;

pub enum Object {
    ObjString(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::ObjString(s) => write!(f, "{}", s),
        }
    }
}
