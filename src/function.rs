use crate::mapping::{Mapping};

#[derive(Debug, Copy, Clone)]
pub enum Func {
    Turbo {
        on_press: Option<Mapping>,
        on_release: Option<Mapping>,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Function {
    pub value: f64,
    state: u8,
    func: Func,
}

impl Function {
    pub fn new(func: Func) -> Self {
        Self {
            value: 0.0,
            state: 0,
            func: func,
        }
    }
}
