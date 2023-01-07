mod frame;
mod run;

use self::frame::Frame;
use crate::{
    compiler::ByteCode,
    object::{
        objects::{Boolean, CompiledFunctionObj, Null},
        AllObjects,
    },
};
use anyhow::{anyhow, Result};

/// Maximum number of objects that can be at a given time in the stack
const STACK_SIZE: usize = 2048;

/// Maximum number of stack frames that can exist at a given time
const MAX_FRAMES: usize = 1024;

/// TRUE constant
const TRUE: AllObjects = AllObjects::Boolean(Boolean { value: true });

/// FALSE constant
const FALSE: AllObjects = AllObjects::Boolean(Boolean { value: false });

/// NULL constant
const NULL: AllObjects = AllObjects::Null(Null);

pub struct VM {
    /// the constants list obtained from the bytecode
    constants: Vec<AllObjects>,

    /// stack will host the operands and the results
    stack: Vec<AllObjects>,

    /// holder of global variable objects
    pub globals: Vec<AllObjects>,

    /// last popped stack element
    result: Option<AllObjects>,

    /// stack frames created for all functions including main
    frames: Vec<Frame>,

    /// current active frame
    frames_index: usize,
}

impl VM {
    /// Creates a new VM using the provided bytecode
    pub fn new(bytecode: ByteCode) -> Self {
        let main_fn = CompiledFunctionObj::new(bytecode.instructions, 0);
        let main_frame = Frame::new(main_fn, vec![]);

        let mut frames = Vec::with_capacity(MAX_FRAMES);
        frames.push(main_frame);

        Self {
            constants: bytecode.constants,
            stack: Vec::with_capacity(STACK_SIZE),
            globals: Vec::new(),
            result: None,
            frames,
            frames_index: 1,
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
        if self.stack.len() >= STACK_SIZE {
            return Err(anyhow!("stack overflow"));
        }
        self.stack.push(val);
        Ok(())
    }

    /// Removes the last value from the stack and returns it after decrementing the stack pointer.
    ///
    /// If the stack is empty after this call and the instructions are empty, this also sets the final result to be returned.
    fn pop(&mut self) -> Result<AllObjects> {
        let Some(obj)  = self.stack.pop() else {
            return Err(anyhow!("stack is empty"));
        };

        if self.stack.is_empty()
            && (self.current_frame().ip + 1) >= self.current_frame().instructions().len()
        {
            self.result = Some(obj.clone());
        }

        Ok(obj)
    }

    fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frames_index - 1]
    }

    fn push_frame(&mut self, f: Frame) {
        self.frames.push(f);
        self.frames_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frames_index -= 1;
        self.frames.pop().unwrap()
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
            (
                "let fivePlusTen = fn() { 5 + 10; };
                 let result = fivePlusTen() + 20; result;",
                Int(35),
            ),
            (
                "let one = fn() { 1; };
                let two = fn() { 2; };
                one() + two()",
                Int(3),
            ),
            (
                "let a = fn() { 1 };
                 let b = fn() { a() + 1 };
                 let c = fn() { b() + 1 };
                 c();",
                Int(3),
            ),
            (
                "let earlyExit = fn() { return 99; 100; };
            earlyExit();",
                Int(99),
            ),
            (
                "let earlyExit = fn() { return 99; return 100; };
            earlyExit();",
                Int(99),
            ),
            (
                "let noReturn = fn() { };
            noReturn();",
                Literal::Null,
            ),
            (
                "let noReturn = fn() { };
                let noReturnTwo = fn() { noReturn(); };
                noReturnTwo();",
                Literal::Null,
            ),
            (
                "
            let returnsOne = fn() { 1; };
            let returnsOneReturner = fn() { returnsOne; };
            returnsOneReturner()();",
                Int(1),
            ),
            (
                "let one = fn() { let one = 1; one };
            one();",
                Int(1),
            ),
            (
                "let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
            oneAndTwo();",
                Int(3),
            ),
            (
                "let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
            let threeAndFour = fn() { let three = 3; let four = 4; three + four; };
            oneAndTwo() + threeAndFour();",
                Int(10),
            ),
            (
                "let firstFoobar = fn() { let foobar = 50; foobar; };
            let secondFoobar = fn() { let foobar = 100; foobar; };
            firstFoobar() + secondFoobar();",
                Int(150),
            ),
            (
                "let globalSeed = 50;
                let minusOne = fn() {
                    let num = 1;
                    globalSeed - num;
                }
                let minusTwo = fn() {
                    let num = 2;
                    globalSeed - num;
                }
                minusOne() + minusTwo();",
                Int(97),
            ),
            (
                "let returnsOneReturner = fn() {
                    let returnsOne = fn() { 1; };
                    returnsOne;
                };
                returnsOneReturner()();",
                Int(1),
            ),
            (
                "let x = 10;
                 x = 20;
                 x;",
                Int(20),
            ),
            (
                "let x = 10;
                 fn() {
                    x = 30 + 50;
                    x;
                 }()",
                Int(80),
            ),
            (
                "let identity = fn(a) { a; };
                identity(4);",
                Int(4),
            ),
            (
                "let sum = fn(a, b) { a + b; };
                sum(1, 2);",
                Int(3),
            ),
            (
                "let calc = fn(a, b) {
                    let c = a + b;
                    let d = 10;
                    d - c;
                };
                calc(1, 2);",
                Int(7),
            ),
            (
                "let sum = fn(a, b) {
                    let c = a + b;
                    c; 
                };
                sum(1, 2) + sum(3, 4);",
                Int(10),
            ),
            (
                "let sum = fn(a, b) {
                   let c = a + b;
                   c; 
                };
                let outer = fn() {
                    sum(1, 2) + sum(3, 4);
                };
                outer();",
                Int(10),
            ),
            (
                "let globalNum = 10;
                 let sum = fn(a, b) {
                    let c = a + b;
                    c + globalNum;
                 };
                let outer = fn() {
                   sum(1, 2) + sum(3, 4) + globalNum;
                };
                outer() + globalNum;",
                Int(50),
            ),
        ];
        let num_test_cases = test_cases.len();

        for (i, tc) in test_cases.into_iter().enumerate() {
            let program = parse(tc.0);
            let mut comp = Compiler::new();
            if let Err(e) = comp.compile(program.make_node()) {
                panic!("input: {}, compiler error:  {}", tc.0, e);
            }

            let mut vm = VM::new(comp.byte_code());
            // for debugging
            if i == num_test_cases - 1 {
                if let Err(e) = vm.run() {
                    panic!("input: {}, vm error:  {}", tc.0, e);
                }
            } else {
                if let Err(e) = vm.run() {
                    panic!("input: {}, vm error:  {}", tc.0, e);
                }
            }

            let stack_elem = vm.result();
            test_expected_object(tc.1, stack_elem.unwrap());
        }
    }

    #[test]
    fn test_vm_fails() {
        let test_cases = vec![
            (
                "fn() { 1; }(1);",
                "wrong number of arguments: want=0, got=1",
            ),
            (
                "fn(a) { a; }();",
                "wrong number of arguments: want=1, got=0",
            ),
            (
                "fn(a, b) { a + b; }(1);",
                "wrong number of arguments: want=2, got=1",
            ),
        ];

        for tc in test_cases {
            let program = parse(tc.0);
            let mut comp = Compiler::new();
            if let Err(e) = comp.compile(program.make_node()) {
                panic!("input: {}, compiler error:  {}", tc.0, e);
            }
            let mut vm = VM::new(comp.byte_code());
            if let Err(e) = vm.run() {
                assert_eq!(e.to_string(), tc.1);
            } else {
                panic!("expected the program to fail with the given error.")
            }
        }
    }
}
