use crate::{
    code::{self, Instructions, OP_ADD, OP_CONSTANT},
    compiler::ByteCode,
    object::{objects::Integer, AllObjects, Object},
};
use anyhow::{anyhow, Result};

/// Maximum number of instruction pointers that can be at a given time in the stack
const STACK_SIZE: usize = 2048;

pub struct VM {
    /// the constants list obtained from the bytecode
    pub constants: Vec<AllObjects>,

    /// bytecode instructions
    pub instructions: Instructions,

    /// stack contains the operands and the results
    pub stack: Vec<AllObjects>,

    /// stack pointer, which always points to the next value. Top of stack is stack[sp-1]
    sp: usize,
}

impl VM {
    /// Creates a new VM using the provided bytecode
    pub fn new(bytecode: ByteCode) -> Self {
        Self {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    /// Runs all the bytecode instructions.
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut ip = 0;

        while ip < self.instructions.len() {
            let op = self.instructions[ip];
            match op {
                OP_CONSTANT => {
                    let const_index = code::helpers::read_u16(&self.instructions[(ip + 1)..]);
                    if self.constants.get(const_index).is_none() {
                        return Err(anyhow!("constant at the index {const_index} not found"));
                    }
                    self.push(self.constants[const_index].clone())?;
                    ip += 2;
                }
                OP_ADD => {
                    let right_value = match self.pop()? {
                        AllObjects::Integer(v) => v,
                        v => return Err(anyhow!("expected an INTEGER, found {}", v.inspect())),
                    };
                    let left_value = match self.pop()? {
                        AllObjects::Integer(v) => v,
                        v => return Err(anyhow!("expected an INTEGER, FOUND {}", v.inspect())),
                    };
                    self.push(AllObjects::Integer(Integer {
                        value: left_value.value + right_value.value,
                    }))?;
                }
                _ => {}
            }
            ip += 1;
        }

        Ok(())
    }

    /// Return the top most element from the stack.
    pub fn stack_top(&self) -> Option<&AllObjects> {
        self.stack.get(self.sp - 1)
    }

    /// Pushes the given object on to the stack and increments the stack pointer.
    fn push(&mut self, val: AllObjects) -> Result<()> {
        if self.sp >= STACK_SIZE {
            return Err(anyhow!("stack overflow"));
        }
        self.stack.push(val);
        self.sp += 1;
        Ok(())
    }

    /// Removes the last inserted constant index from the stack and returns it after decrementing the stack pointer.
    fn pop(&mut self) -> Result<AllObjects> {
        let Some(obj)  = self.stack.pop() else {
            return Err(anyhow!("stack is empty"));
        };
        self.sp -= 1;
        Ok(obj)
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
        let test_cases = vec![("11", Int(11)), ("27", Int(27)), ("13 + 29", Int(42))];

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
