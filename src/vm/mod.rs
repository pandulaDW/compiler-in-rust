#![allow(dead_code)]

use crate::{
    code::{self, Instructions, OP_CONSTANT},
    compiler::ByteCode,
    object::AllObjects,
};
use anyhow::anyhow;

/// Maximum number of instruction pointers that can be at a given time in the stack
const STACK_SIZE: usize = 2048;

struct VM {
    /// the constants list obtained from the bytecode
    pub constants: Vec<AllObjects>,

    /// bytecode instructions
    pub instructions: Instructions,

    /// contains indices to the constants (to avoid cloning the constants)
    pub stack: Vec<usize>,

    /// stack pointer, which always points to the next value. Top of stack is stack[sp-1]
    sp: usize,
}

impl VM {
    /// Creates a new VM using the provided bytecode
    fn new(bytecode: ByteCode) -> Self {
        Self {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    /// Return the top most element from the stack.
    fn stack_top(&self) -> Option<&AllObjects> {
        let const_index = self.stack.get(self.sp - 1)?.to_owned();
        self.constants.get(const_index)
    }

    /// Runs all the bytecode instructions.
    fn run(&mut self) -> anyhow::Result<()> {
        let mut ip = 0;

        while ip < self.instructions.len() {
            let op = self.instructions[ip];
            match op {
                OP_CONSTANT => {
                    let const_index = code::helpers::read_u16(&self.instructions[(ip + 1)..]);
                    ip += 2;
                    if let Err(e) = self.push(const_index) {
                        return Err(e);
                    }
                }
                _ => {}
            }
            ip += 1;
        }

        Ok(())
    }

    /// Pushes the given object on to the stack.
    fn push(&mut self, const_index: usize) -> anyhow::Result<()> {
        if self.sp >= STACK_SIZE {
            return Err(anyhow!("stack overflow"));
        }
        if self.constants.get(const_index).is_none() {
            return Err(anyhow!("constant at the index {const_index} not found"));
        }
        self.stack.push(const_index);
        self.sp += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::{
            test_helpers::{parse, test_integer_object, Literal},
            Compiler,
        },
        object::AllObjects,
        vm::VM,
    };

    #[test]
    fn test_vm_works() {
        use Literal::Int;

        // input, expected
        let test_cases = vec![("11", Int(11)), ("27", Int(27)), ("13 + 29", Int(29))];

        for tc in test_cases {
            let program = parse(tc.0);
            let mut comp = Compiler::new();
            if let Err(e) = comp.compile(program.make_node()) {
                panic!("compiler error:  {}", e);
            }

            let mut vm = VM::new(comp.byte_code());
            if let Err(e) = vm.run() {
                panic!("vm error:  {}", e);
            }

            let stack_elem = vm.stack_top();
            helper_test_expected_object(tc.1, stack_elem.unwrap());
        }
    }

    fn helper_test_expected_object(expected: Literal, actual: &AllObjects) {
        match expected {
            Literal::Int(v) => test_integer_object(v, actual),
        }
    }
}
