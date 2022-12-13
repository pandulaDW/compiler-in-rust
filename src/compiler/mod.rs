mod compile;
mod symbol_table;

use crate::{
    code::{self, make, Opcode},
    object::AllObjects,
};

use self::symbol_table::SymbolTable;

#[derive(Default, Clone)]
struct EmittedInstruction {
    opcode: Opcode,
    _position: usize,
}

pub struct Compiler {
    /// instructions will hold the generated bytecode
    instructions: code::Instructions,

    /// constants is a slice that serves as our constant pool.
    constants: Vec<AllObjects>,

    /// very last instruction emitted
    last_instruction: EmittedInstruction,

    /// symbol table for all scopes
    symbol_table: SymbolTable,

    /// the instruction before the last instruction
    previous_instruction: EmittedInstruction,
}

impl Compiler {
    /// Creates a new compiler with empty instructions and constant pool.
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            constants: vec![],
            last_instruction: EmittedInstruction::default(),
            symbol_table: SymbolTable::new(),
            previous_instruction: EmittedInstruction::default(),
        }
    }

    /// Emits the byte-code instructions after compilation has finished.
    pub fn byte_code(self) -> ByteCode {
        ByteCode {
            instructions: self.instructions,
            constants: self.constants,
        }
    }

    /// Generates an instruction and add it to the results and the starting position of the
    /// just-emitted instruction will be returned.
    fn emit(&mut self, op: Opcode, operands: &[usize]) -> usize {
        let mut instructions = make(op, operands);
        let current_position = self.instructions.len();
        self.instructions.append(&mut instructions);

        self.set_last_instruction(op, current_position);

        current_position
    }

    /// Set the last instruction and the last-to-previous instruction
    fn set_last_instruction(&mut self, opcode: Opcode, position: usize) {
        self.previous_instruction = self.last_instruction.clone();
        self.last_instruction = EmittedInstruction {
            opcode,
            _position: position,
        };
    }

    /// Removes the last pop instruction
    fn remove_last_pop(&mut self) {
        self.instructions.pop();
        self.last_instruction = self.previous_instruction.clone();
    }

    /// Add the given constant to the constant pool and return it's index position.
    fn add_constant(&mut self, obj: AllObjects) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }
}

/// Bytecode is what gets pass to the VM
pub struct ByteCode {
    pub instructions: code::Instructions,
    pub constants: Vec<AllObjects>,
}

#[cfg(test)]
mod tests {
    use super::code::*;
    use super::test_helpers::*;

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
            // (
            //     "let one = 1;
            //     let two = 2;",
            //     vec![Int(1), Int(2)],
            //     vec![
            //         make(OP_CONSTANT, &[0]),
            //         make(OP_SET_GLOBAL, &[0]),
            //         make(OP_CONSTANT, &[1]),
            //         make(OP_SET_GLOBAL, &[1]),
            //     ],
            // ),
            // (
            //     "let one = 1;
            //     one;",
            //     vec![Int(1)],
            //     vec![
            //         make(OP_CONSTANT, &[0]),
            //         make(OP_SET_GLOBAL, &[0]),
            //         make(OP_GET_GLOBAL, &[0]),
            //         make(OP_POP, &[]),
            //     ],
            // ),
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
}

#[cfg(test)]
pub mod test_helpers {
    use super::{code::Instructions, Compiler};
    use crate::{ast::program::Program, lexer::Lexer, object::AllObjects, parser::Parser};

    pub enum Literal {
        Int(i64),
        Bool(bool),
        Null,
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
            test_instructions(tc.2, bytecode.instructions);
            test_constants(tc.1, bytecode.constants);
        }
    }

    fn test_instructions(expected: Vec<Instructions>, actual: Instructions) {
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
                Literal::Bool(_) => {}
                Literal::Null => {}
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

    pub fn parse(input: &str) -> Program {
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        p.parse_program()
    }
}
