mod compile;
mod symbol_table;

use crate::{
    code::{self, make, Instructions, Opcode},
    object::AllObjects,
};

pub use self::symbol_table::SymbolTable;

#[derive(Default, Clone)]
struct EmittedInstruction {
    opcode: Opcode,
    position: usize,
}

impl EmittedInstruction {
    fn new(opcode: Opcode, position: usize) -> Self {
        Self { opcode, position }
    }
}

pub struct Compiler {
    /// instructions will hold the generated bytecode
    instructions: code::Instructions,

    /// constants is a slice that serves as our constant pool.
    pub constants: Vec<AllObjects>,

    /// very last instruction emitted
    last_instruction: EmittedInstruction,

    /// symbol table for all scopes
    pub symbol_table: SymbolTable,

    /// the instruction before the last instruction
    previous_instruction: EmittedInstruction,

    /// contains all the scopes that would be encountered in the compilation process
    scopes: Vec<CompilationScope>,

    /// current active scope index
    scope_index: usize,
}

impl Compiler {
    /// Creates a new compiler with empty instructions and constant pool.
    pub fn new() -> Self {
        let main_scope = CompilationScope {
            instructions: vec![],
            last_instructions: EmittedInstruction::default(),
            previous_instruction: EmittedInstruction::default(),
        };

        Self {
            instructions: vec![],
            constants: vec![],
            last_instruction: EmittedInstruction::default(),
            symbol_table: SymbolTable::new(),
            previous_instruction: EmittedInstruction::default(),
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    /// Creates a new compiler with the given state (for the REPL)
    pub fn new_with_state(symbol_table: SymbolTable, constants: Vec<AllObjects>) -> Self {
        Self {
            instructions: vec![],
            constants,
            last_instruction: EmittedInstruction::default(),
            symbol_table,
            previous_instruction: EmittedInstruction::default(),
            scopes: vec![],
            scope_index: 0,
        }
    }

    /// Emits the byte-code instructions after compilation has finished.
    pub fn byte_code(mut self) -> ByteCode {
        ByteCode {
            instructions: self.current_instructions().clone(),
            constants: self.constants,
        }
    }

    /// Generates an instruction and add it to the current scope and updates the last instruction.
    /// the position of the just-emitted instruction will be returned.
    fn emit(&mut self, op: Opcode, operands: &[usize]) -> usize {
        let instructions = make(op, operands);
        let pos_new_instruction = self.current_instructions().len();
        self.current_instructions().extend_from_slice(&instructions);

        self.set_last_instruction(op, pos_new_instruction);
        pos_new_instruction
    }

    /// Create a new scope and make it active
    fn enter_scope(&mut self) {
        let scope = CompilationScope {
            instructions: vec![],
            last_instructions: EmittedInstruction::default(),
            previous_instruction: EmittedInstruction::default(),
        };
        self.scopes.push(scope);
        self.scope_index += 1;
    }

    /// Removed the created scope and make the second-to-last one active
    fn leave_scope(&mut self) -> Instructions {
        let instructions = self.current_instructions().clone();
        self.scopes.pop();
        self.scope_index -= 1;
        instructions
    }

    /// Set the last instruction and the last-to-previous instruction
    fn set_last_instruction(&mut self, opcode: Opcode, position: usize) {
        let previous = self.scopes[self.scope_index].last_instructions.clone();
        let last = EmittedInstruction::new(opcode, position);

        self.scopes[self.scope_index].previous_instruction = previous;
        self.scopes[self.scope_index].last_instructions = last;
    }

    /// Removes the last pop instruction
    fn remove_last_pop(&mut self) {
        let last = self.scopes[self.scope_index].last_instructions.clone();
        let previous = self.scopes[self.scope_index].previous_instruction.clone();

        let old = self.current_instructions();
        let new = &old[..last.position];

        self.scopes[self.scope_index].instructions = new.to_vec();
        self.scopes[self.scope_index].last_instructions = previous;
    }

    /// Add the given constant to the constant pool and return it's index position.
    fn add_constant(&mut self, obj: AllObjects) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    /// Return the instruction set of the current active scope
    fn current_instructions(&mut self) -> &mut code::Instructions {
        &mut self.scopes[self.scope_index].instructions
    }

    /// Check if the last instruction is a POP instruction
    fn last_instruction_is_pop(&self) -> bool {
        self.scopes[self.scope_index].last_instructions.opcode == code::OP_POP
    }
}

/// Bytecode is what gets pass to the VM
pub struct ByteCode {
    pub instructions: code::Instructions,
    pub constants: Vec<AllObjects>,
}

struct CompilationScope {
    instructions: code::Instructions,
    last_instructions: EmittedInstruction,
    previous_instruction: EmittedInstruction,
}

#[cfg(test)]
mod tests {
    use super::code::*;
    use super::test_helpers::*;
    use super::Compiler;

    #[test]
    fn test_integer_arithmetic() {
        use Literal::Int;

        let test_cases = vec![
            (
                "1; 2",
                vec![Int(1), Int(2)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_POP, &[]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "11 + 25",
                vec![Int(11), Int(25)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_ADD, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "13 - 18",
                vec![Int(13), Int(18)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_SUB, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "7 * 8",
                vec![Int(7), Int(8)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_MUL, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "2 / 1",
                vec![Int(2), Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_DIV, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "-81",
                vec![Int(81)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_MINUS, &[]),
                    make(OP_POP, &[]),
                ],
            ),
        ];

        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_boolean_expressions() {
        use Literal::Int;
        let test_cases: Vec<CompilerTestCase> = vec![
            ("true", vec![], vec![make(OP_TRUE, &[]), make(OP_POP, &[])]),
            (
                "false",
                vec![],
                vec![make(OP_FALSE, &[]), make(OP_POP, &[])],
            ),
            (
                "1 > 2",
                vec![Int(1), Int(2)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_GREATER_THAN, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "1 < 2",
                vec![Int(2), Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_GREATER_THAN, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "1 == 2",
                vec![Int(1), Int(2)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_EQUAL, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "1 != 2",
                vec![Int(1), Int(2)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_NOT_EQUAL, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "true == false",
                vec![],
                vec![
                    make(OP_TRUE, &[]),
                    make(OP_FALSE, &[]),
                    make(OP_EQUAL, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "true != false",
                vec![],
                vec![
                    make(OP_TRUE, &[]),
                    make(OP_FALSE, &[]),
                    make(OP_NOT_EQUAL, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "!true",
                vec![],
                vec![make(OP_TRUE, &[]), make(OP_BANG, &[]), make(OP_POP, &[])],
            ),
        ];
        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_conditionals() {
        use Literal::Int;

        let test_cases: Vec<CompilerTestCase> = vec![
            (
                "if (true) { 10; }; 3333;",
                vec![Int(10), Int(3333)],
                vec![
                    make(OP_TRUE, &[]),              // 0000
                    make(OP_JUMP_NOT_TRUTHY, &[10]), // 0001
                    make(OP_CONSTANT, &[0]),         // 0004
                    make(OP_JUMP, &[11]),            // 0007
                    make(OP_NULL, &[]),              // 0010
                    make(OP_POP, &[]),               // 0011
                    make(OP_CONSTANT, &[1]),         // 0012
                    make(OP_POP, &[]),               // 0015
                ],
            ),
            (
                "if (true) { 10; } else { 20 }; 3333;",
                vec![Int(10), Int(20), Int(3333)],
                vec![
                    make(OP_TRUE, &[]),              // 0000
                    make(OP_JUMP_NOT_TRUTHY, &[10]), // 0001
                    make(OP_CONSTANT, &[0]),         // 0004
                    make(OP_JUMP, &[13]),            // 0007
                    make(OP_CONSTANT, &[1]),         // 0010
                    make(OP_POP, &[]),               // 0013
                    make(OP_CONSTANT, &[2]),         // 0014
                    make(OP_POP, &[]),               // 0017
                ],
            ),
        ];
        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_global_let_statements() {
        use Literal::Int;

        let test_cases: Vec<CompilerTestCase> = vec![
            (
                "let one = 1;
                let two = 2;",
                vec![Int(1), Int(2)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_SET_GLOBAL, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_SET_GLOBAL, &[1]),
                ],
            ),
            (
                "let one = 1;
                one;",
                vec![Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_SET_GLOBAL, &[0]),
                    make(OP_GET_GLOBAL, &[0]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "let one = 1;
                let two = one;
                two;",
                vec![Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_SET_GLOBAL, &[0]),
                    make(OP_GET_GLOBAL, &[0]),
                    make(OP_SET_GLOBAL, &[1]),
                    make(OP_GET_GLOBAL, &[1]),
                    make(OP_POP, &[]),
                ],
            ),
        ];
        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_string_expressions() {
        use Literal::Str;

        let test_cases: Vec<CompilerTestCase> = vec![
            (
                r#" "monkey" "#,
                vec![Str("monkey")],
                vec![make(OP_CONSTANT, &[0]), make(OP_POP, &[])],
            ),
            (
                r#" "mon" + "key" "#,
                vec![Str("mon"), Str("key")],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_ADD, &[]),
                    make(OP_POP, &[]),
                ],
            ),
        ];

        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_array_literals() {
        use Literal::Int;

        let test_cases: Vec<CompilerTestCase> = vec![
            ("[]", vec![], vec![make(OP_ARRAY, &[0]), make(OP_POP, &[])]),
            (
                "[1, 2, 3]",
                vec![Int(1), Int(2), Int(3)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_ARRAY, &[3]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "[1 + 2, 3 - 4, 5 * 6]",
                vec![Int(1), Int(2), Int(3), Int(4), Int(5), Int(6)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_ADD, &[]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_CONSTANT, &[3]),
                    make(OP_SUB, &[]),
                    make(OP_CONSTANT, &[4]),
                    make(OP_CONSTANT, &[5]),
                    make(OP_MUL, &[]),
                    make(OP_ARRAY, &[3]),
                    make(OP_POP, &[]),
                ],
            ),
        ];

        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_hash_literals() {
        use Literal::Int;
        let test_cases: Vec<CompilerTestCase> = vec![
            ("{}", vec![], vec![make(OP_HASH, &[0]), make(OP_POP, &[])]),
            (
                "{1: 2, 3: 4, 5: 6}",
                vec![Int(1), Int(2), Int(3), Int(4), Int(5), Int(6)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_CONSTANT, &[3]),
                    make(OP_CONSTANT, &[4]),
                    make(OP_CONSTANT, &[5]),
                    make(OP_HASH, &[3]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "{1: 2 + 3, 4: 5 * 6}",
                vec![Int(1), Int(2), Int(3), Int(4), Int(5), Int(6)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_ADD, &[]),
                    make(OP_CONSTANT, &[3]),
                    make(OP_CONSTANT, &[4]),
                    make(OP_CONSTANT, &[5]),
                    make(OP_MUL, &[]),
                    make(OP_HASH, &[2]),
                    make(OP_POP, &[]),
                ],
            ),
        ];
        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_index_expressions() {
        use Literal::Int;
        let test_cases: Vec<CompilerTestCase> = vec![
            (
                "[1, 2, 3][1 + 1]",
                vec![Int(1), Int(2), Int(3), Int(1), Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_ARRAY, &[3]),
                    make(OP_CONSTANT, &[3]),
                    make(OP_CONSTANT, &[4]),
                    make(OP_ADD, &[]),
                    make(OP_INDEX, &[]),
                    make(OP_POP, &[]),
                ],
            ),
            (
                "{1: 2}[2 - 1]",
                vec![Int(1), Int(2), Int(2), Int(1)],
                vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_HASH, &[1]),
                    make(OP_CONSTANT, &[2]),
                    make(OP_CONSTANT, &[3]),
                    make(OP_SUB, &[]),
                    make(OP_INDEX, &[]),
                    make(OP_POP, &[]),
                ],
            ),
        ];
        run_compiler_tests(test_cases);
    }

    #[test]
    #[ignore = "reason"]
    fn test_function_literals() {
        use Literal::{Ins, Int};
        let test_cases: Vec<CompilerTestCase> = vec![(
            "fn() { return 5 + 10 }",
            vec![
                Int(5),
                Int(10),
                Ins(vec![
                    make(OP_CONSTANT, &[0]),
                    make(OP_CONSTANT, &[1]),
                    make(OP_ADD, &[]),
                    make(OP_RETURN_VALUE, &[]),
                ]),
            ],
            vec![make(OP_CONSTANT, &[2]), make(OP_POP, &[])],
        )];
        run_compiler_tests(test_cases);
    }

    #[test]
    fn test_compiler_scopes() {
        let mut compiler = Compiler::new();
        assert_eq!(compiler.scope_index, 0);

        compiler.emit(OP_MUL, &[]);
        assert_eq!(compiler.scopes[0].instructions.len(), 1);
        assert_eq!(compiler.scopes[0].last_instructions.opcode, OP_MUL);

        compiler.enter_scope();
        assert_eq!(compiler.scope_index, 1);

        compiler.emit(OP_SUB, &[]);
        assert_eq!(compiler.scopes[1].instructions.len(), 1);
        assert_eq!(compiler.scopes[1].last_instructions.opcode, OP_SUB);

        compiler.leave_scope();
        assert_eq!(compiler.scopes.len(), 1);
        assert_eq!(compiler.scope_index, 0);

        compiler.emit(OP_ADD, &[]);
        assert_eq!(compiler.scopes[0].instructions.len(), 2);
        assert_eq!(compiler.scopes[0].last_instructions.opcode, OP_ADD);
        assert_eq!(compiler.scopes[0].previous_instruction.opcode, OP_MUL);
    }
}

#[cfg(test)]
pub mod test_helpers {
    use std::{collections::HashMap, fmt::Display, hash::Hash};

    use super::{code::Instructions, Compiler};
    use crate::{
        ast::program::Program,
        lexer::Lexer,
        object::{AllObjects, Object},
        parser::Parser,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Literal {
        Int(i64),
        Bool(bool),
        Str(&'static str),
        Arr(Vec<Literal>),
        Hash(HashMap<Literal, Literal>),
        Null,
        Ins(Vec<Instructions>),
    }

    impl Display for Literal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let out = match self {
                Self::Int(v) => v.to_string(),
                Self::Bool(v) => v.to_string(),
                Self::Str(v) => v.to_string(),
                Self::Arr(v) => format!("{:?}", v),
                Self::Hash(v) => format!("{:?}", v),
                Self::Null => "null".to_string(),
                Self::Ins(v) => format!("{:?}", v),
            };
            write!(f, "{out}")
        }
    }

    impl Hash for Literal {
        fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
            match self {
                Literal::Int(v) => v.to_string(),
                Literal::Bool(v) => v.to_string(),
                Literal::Str(v) => v.to_string(),
                Literal::Arr(_) => unimplemented!(),
                Literal::Hash(_) => unimplemented!(),
                Literal::Null => unimplemented!(),
                Literal::Ins(_) => unimplemented!(),
            };
        }
    }

    // input, expectedConstants, expectedInstructions
    pub type CompilerTestCase<'a> = (&'a str, Vec<Literal>, Vec<Instructions>);

    pub fn run_compiler_tests(test_cases: Vec<CompilerTestCase>) {
        for tc in test_cases {
            let program = parse(tc.0);
            let mut compiler = Compiler::new();
            if let Err(e) = compiler.compile(program.make_node()) {
                panic!("compiler error: {e}");
            };

            let bytecode = compiler.byte_code();
            test_instructions(tc.2, &bytecode.instructions);
            test_constants(tc.1, bytecode.constants);
        }
    }

    fn test_instructions(expected: Vec<Instructions>, actual: &Instructions) {
        let concatted = concat_instructions(expected);
        assert_eq!(concatted.len(), actual.len());

        for (i, ins) in concatted.into_iter().enumerate() {
            assert_eq!(actual[i], ins);
        }
    }

    fn test_constants(expected: Vec<Literal>, actual: Vec<AllObjects>) {
        assert_eq!(expected.len(), actual.len());

        for (i, constant) in expected.into_iter().enumerate() {
            match constant {
                Literal::Int(v) => test_integer_object(v, &actual[i]),
                Literal::Str(v) => test_string_object(v, &actual[i]),
                Literal::Bool(v) => test_boolean_object(v, &actual[i]),
                Literal::Arr(v) => test_array_literal(v, &actual[i]),
                Literal::Hash(mut v) => test_hash_literal(&mut v, &actual[i]),
                Literal::Null => test_null_object(&actual[i]),
                Literal::Ins(v) => test_fn_instructions(v, &actual[i]),
            }
        }
    }

    pub fn concat_instructions(s: Vec<Instructions>) -> Instructions {
        s.iter().fold(Vec::new(), |mut out, ins| {
            out.extend_from_slice(ins);
            out
        })
    }

    pub fn test_integer_object(expected: i64, actual: &AllObjects) {
        match actual {
            AllObjects::Integer(v) => assert_eq!(v.value, expected),
            _ => panic!("expected an integer object"),
        };
    }

    pub fn test_string_object(expected: &str, actual: &AllObjects) {
        match actual {
            AllObjects::StringObj(v) => assert_eq!(*v.value, expected),
            _ => panic!("expected an integer object"),
        }
    }

    pub fn test_boolean_object(expected: bool, actual: &AllObjects) {
        match actual {
            AllObjects::Boolean(v) => assert_eq!(v.value, expected),
            _ => panic!("expected a boolean object"),
        };
    }

    pub fn test_null_object(actual: &AllObjects) {
        match actual {
            AllObjects::Null(_) => {}
            _ => panic!("expected a null object"),
        }
    }

    pub fn test_array_literal(expected: Vec<Literal>, actual: &AllObjects) {
        let arr = match actual {
            AllObjects::ArrayObj(v) => v,
            _ => panic!("expected an array object"),
        };

        let elements = arr.elements.borrow();

        for (i, expected_el) in expected.into_iter().enumerate() {
            let Some(actual_el) = elements.get(i) else {
                panic!("element at {i} should exist");
            };
            test_expected_object(expected_el, actual_el);
        }
    }

    pub fn test_hash_literal(expected: &mut HashMap<Literal, Literal>, actual: &AllObjects) {
        let map_obj = match actual {
            AllObjects::HashMap(v) => v,
            _ => panic!("expected a hash literal"),
        };

        let map = map_obj.map.borrow();
        let mut actual_keys: Vec<String> = map.keys().map(|key| key.inspect()).collect();
        actual_keys.sort();

        let mut expected_keys: Vec<String> = expected.keys().map(|v| v.to_string()).collect();
        expected_keys.sort();

        assert_eq!(actual_keys, expected_keys);

        for key in map.keys() {
            let actual_value = &map[key];
            let expected_key = expected
                .keys()
                .find(|v| v.to_string() == key.inspect())
                .unwrap(); // checked previously
            let expected_value = expected[expected_key].clone();
            test_expected_object(expected_value, actual_value);
        }
    }

    pub fn test_fn_instructions(expected: Vec<Instructions>, actual: &AllObjects) {
        let actual_ins = match actual {
            AllObjects::CompiledFunction(v) => v,
            _ => panic!("expected fn instructions"),
        };
        test_instructions(expected, &actual_ins.instructions);
    }

    pub fn parse(input: &str) -> Program {
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        p.parse_program()
    }

    pub fn test_expected_object(expected: Literal, actual: &AllObjects) {
        match expected {
            Literal::Int(v) => test_integer_object(v, actual),
            Literal::Bool(v) => test_boolean_object(v, actual),
            Literal::Str(v) => test_string_object(v, actual),
            Literal::Arr(v) => test_array_literal(v, actual),
            Literal::Hash(mut v) => test_hash_literal(&mut v, actual),
            Literal::Null => test_null_object(actual),
            Literal::Ins(_) => {}
        }
    }
}
