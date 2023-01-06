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
    /// Create a new frame with the compiled function and a arguments vector as the initial
    /// locals list.
    pub fn new(func: CompiledFunctionObj, arguments: Vec<AllObjects>) -> Self {
        Self {
            func,
            ip: 0,
            locals: arguments,
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
