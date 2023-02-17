use crate::string::LoxStr;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: LoxStr,
}

impl LoxClass {
    pub fn new(name: LoxStr) -> Self {
        return Self { name };
    }
}
