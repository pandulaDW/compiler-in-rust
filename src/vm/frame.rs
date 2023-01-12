use crate::{
    code::Instructions,
    object::{objects::Closure, AllObjects},
};

#[derive(Clone)]
pub struct Frame {
    /// Compiled closure object which also contains the function
    pub closure: Closure,

    /// instruction pointer, which points the index of the currently executing opcode
    pub ip: usize,

    /// holder of local variable objects
    pub locals: Vec<AllObjects>,
}

impl Frame {
    /// Create a new frame with the compiled function and an arguments vector as the initial
    /// locals list.
    pub fn new(closure: Closure, arguments: Vec<AllObjects>) -> Self {
        Self {
            closure,
            ip: 0,
            locals: arguments,
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.closure.func.instructions
    }
}
