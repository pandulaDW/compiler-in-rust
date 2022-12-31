use crate::{code::Instructions, object::objects::CompiledFunctionObj};

#[derive(Clone)]
pub struct Frame {
    func: CompiledFunctionObj,

    /// instruction pointer, which points the index of the currently executing opcode
    pub ip: usize,
}

impl Frame {
    pub fn new(func: CompiledFunctionObj) -> Self {
        Self { func, ip: 0 }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
