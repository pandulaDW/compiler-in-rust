mod run;

use crate::{
    code::Instructions,
    compiler::ByteCode,
    object::{
        objects::{Boolean, Null},
        AllObjects,
    },
};
use anyhow::{anyhow, Result};

/// Maximum number of objects that can be at a given time in the stack
const STACK_SIZE: usize = 2048;

/// TRUE constant
const TRUE: AllObjects = AllObjects::Boolean(Boolean { value: true });

/// FALSE constant
const FALSE: AllObjects = AllObjects::Boolean(Boolean { value: false });

/// NULL constant
const NULL: AllObjects = AllObjects::Null(Null);

pub struct VM {
    /// the constants list obtained from the bytecode
    constants: Vec<AllObjects>,

    /// bytecode instructions
    instructions: Instructions,

    /// stack will host the operands and the results
    stack: Vec<AllObjects>,

    /// stack pointer, which always points to the next value. Top of stack is stack[sp-1]
    sp: usize,

    /// instruction pointer, which points the index of the currently executing opcode
    ip: usize,

    /// holder of global variable objects
    pub globals: Vec<AllObjects>,

    /// last popped stack element
    result: Option<AllObjects>,
}

impl VM {
    /// Creates a new VM using the provided bytecode
    pub fn new(bytecode: ByteCode) -> Self {
        Self {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: Vec::with_capacity(STACK_SIZE),
            globals: Vec::new(),
            sp: 0,
            ip: 0,
            result: None,
        }
    }

    /// Creates a new VM with the given global variables (for the REPL)
    pub fn new_with_global_store(bytecode: ByteCode, s: Vec<AllObjects>) -> Self {
        let mut vm = Self::new(bytecode);
        vm.globals = s;
        vm
    }

    /// Return the top most element from the stack.
    pub fn result(&self) -> Option<&AllObjects> {
        self.result.as_ref()
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

    /// Removes the last value from the stack and returns it after decrementing the stack pointer.
    ///
    /// If the stack is empty after this call and the instructions are empty, this also sets the final result to be returned.
    fn pop(&mut self) -> Result<AllObjects> {
        let Some(obj)  = self.stack.pop() else {
            return Err(anyhow!("stack is empty"));
        };
        self.sp -= 1;

        if self.stack.is_empty() && (self.ip + 1) >= self.instructions.len() {
            self.result = Some(obj.clone());
        }

        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        compiler::{test_helpers::*, Compiler},
        vm::VM,
    };

    #[test]
    fn test_vm_works() {
        use Literal::{Arr, Bool, Hash, Int, Str};

        // input, expected
        let test_cases = vec![
            ("11", Int(11)),
            ("13; 27", Int(27)),
            ("13 + 29", Int(42)),
            ("1 + 2 + 4", Int(7)),
            ("1 - 2", Int(-1)),
            ("3 * 4", Int(12)),
            ("4 / 2", Int(2)),
            ("50 / 2 * 2 + 10 - 5", Int(55)),
            ("5 + 5 + 5 + 5 - 10", Int(10)),
            ("2 * 2 * 2 * 2 * 2", Int(32)),
            ("5 * 2 + 10", Int(20)),
            ("5 + 2 * 10", Int(25)),
            ("5 * (2 + 10)", Int(60)),
            ("true", Bool(true)),
            ("false", Bool(false)),
            ("1 < 2", Bool(true)),
            ("1 > 2", Bool(false)),
            ("1 < 1", Bool(false)),
            ("1 > 1", Bool(false)),
            ("1 == 1", Bool(true)),
            ("1 != 1", Bool(false)),
            ("1 == 2", Bool(false)),
            ("1 != 2", Bool(true)),
            ("true == true", Bool(true)),
            ("false == false", Bool(true)),
            ("true == false", Bool(false)),
            ("true != false", Bool(true)),
            ("false != true", Bool(true)),
            ("(1 < 2) == true", Bool(true)),
            ("(1 < 2) == false", Bool(false)),
            ("(1 > 2) == true", Bool(false)),
            ("(1 > 2) == false", Bool(true)),
            ("-5", Int(-5)),
            ("-10", Int(-10)),
            ("-50 + 100 + -50", Int(0)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Int(50)),
            ("!true", Bool(false)),
            ("!false", Bool(true)),
            ("!5", Bool(false)),
            ("!!true", Bool(true)),
            ("!!false", Bool(false)),
            ("!!5", Bool(true)),
            ("if(true) { 10; }", Int(10)),
            ("if (true) { 10 } else { 20 }", Int(10)),
            ("if (false) { 10 } else { 20 } ", Int(20)),
            ("if (1) { 10 }", Int(10)),
            ("if (1 < 2) { 10 }", Int(10)),
            ("if (1 < 2) { 10 } else { 20 }", Int(10)),
            ("if (1 > 2) { 10 } else { 20 }", Int(20)),
            ("if (1 > 2) { 10 }", Literal::Null),
            ("if (false) { 10 }", Literal::Null),
            ("!(if (false) { 5; })", Bool(true)),
            ("if ((if (false) { 10 })) { 10 } else { 20 }", Int(20)),
            ("let one = 1; one", Int(1)),
            ("let one = 1; let two = 2; one + two", Int(3)),
            ("let one = 1; let two = one + one; one + two", Int(3)),
            (r#" "monkey" "#, Str("monkey")),
            (r#" "mon" + "key" "#, Str("monkey")),
            (r#" "mon" + "key" + "banana" "#, Str("monkeybanana")),
            ("[]", Arr(vec![])),
            ("[1, 2, 3]", Arr(vec![Int(1), Int(2), Int(3)])),
            (
                "[1 + 2, 3 - 4, \"foo\", 5 * 6, true]",
                Arr(vec![Int(3), Int(-1), Str("foo"), Int(30), Bool(true)]),
            ),
            ("{}", Hash(HashMap::new())),
            (
                "{1: 2, 3: 4, 5: 6}",
                Hash(
                    vec![(Int(1), Int(2)), (Int(3), Int(4)), (Int(5), Int(6))]
                        .into_iter()
                        .collect(),
                ),
            ),
            (
                "{1: 2 + 3, 4: 5 * 6,true: \"foo\"}",
                Hash(
                    vec![
                        (Int(1), Int(5)),
                        (Int(4), Int(30)),
                        (Bool(true), Str("foo")),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
            ("[1, 2, 3][1]", Int(2)),
            ("[1, 2, 3][0 + 2]", Int(3)),
            ("[[1, 1, 1]][0][0]", Int(1)),
            ("{1: 1, 2: 2}[1]", Int(1)),
            ("{1: 1, 2: 2}[2]", Int(2)),
            ("{1: 1}[0]", Literal::Null),
            ("{}[0]", Literal::Null),
        ];

        for tc in test_cases {
            let program = parse(tc.0);
            let mut comp = Compiler::new();
            if let Err(e) = comp.compile(program.make_node()) {
                panic!("input: {}, compiler error:  {}", tc.0, e);
            }

            let mut vm = VM::new(comp.byte_code());
            if let Err(e) = vm.run() {
                panic!("input: {}, vm error:  {}", tc.0, e);
            }

            let stack_elem = vm.result();
            test_expected_object(tc.1, stack_elem.unwrap());
        }
    }
}
