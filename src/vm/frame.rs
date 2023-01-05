use crate::{
    code::Instructions,
    object::{objects::CompiledFunctionObj, AllObjects},
};

#[derive(Clone)]
pub struct Frame {
    /// Compiled function object
    func: CompiledFunctionObj,

    /// instruction pointer, which points the index of the currently executing opcode
    pub ip: usize,

    /// holder of local variable objects
    pub locals: Vec<AllObjects>,
}

impl Frame {
    pub fn new(func: CompiledFunctionObj) -> Self {
        Self {
            func,
            ip: 0,
            locals: vec![],
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
