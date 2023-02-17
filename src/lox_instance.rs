use crate::lox_class::LoxClass;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    pub class: LoxClass,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        return Self { class };
    }
}
