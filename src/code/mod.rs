pub mod helpers;

use anyhow::anyhow;
use byteorder::{BigEndian, ByteOrder};

/// Opcode is an alias to a byte
pub type Opcode = u8;

/// Instructions are a vector of u8s, which contains all the information needed
/// to carry out an instruction.
pub type Instructions = Vec<Opcode>;

// List of OpCodes which has a width of u8
pub const OP_CONSTANT: Opcode = 1;
pub const OP_ADD: Opcode = 2;
pub const OP_POP: Opcode = 3;
pub const OP_SUB: Opcode = 4;
pub const OP_MUL: Opcode = 5;
pub const OP_DIV: Opcode = 6;
pub const OP_TRUE: Opcode = 7;
pub const OP_FALSE: Opcode = 8;
pub const OP_EQUAL: Opcode = 9;
pub const OP_NOT_EQUAL: Opcode = 10;
pub const OP_GREATER_THAN: Opcode = 11;
pub const OP_MINUS: Opcode = 12;
pub const OP_BANG: Opcode = 13;
pub const OP_JUMP_NOT_TRUTHY: Opcode = 14;
pub const OP_JUMP: Opcode = 15;
pub const OP_NULL: Opcode = 16;
pub const OP_GET_GLOBAL: Opcode = 17;
pub const OP_SET_GLOBAL: Opcode = 18;
pub const OP_ARRAY: Opcode = 19;
pub const OP_HASH: Opcode = 20;
pub const OP_INDEX: Opcode = 21;
pub const OP_CALL: Opcode = 22;
pub const OP_RETURN_VALUE: Opcode = 23;
pub const OP_RETURN: Opcode = 24;
pub const OP_GET_LOCAL: Opcode = 25;
pub const OP_SET_LOCAL: Opcode = 26;
pub const OP_ASSIGN_GLOBAL: Opcode = 27;

/// An opcode definition for debugging and testing purposes
pub struct Definition {
    /// helps to make an Opcode readable
    pub name: String,

    /// contains the number of bytes (width) each operand takes up
    pub operand_widths: Vec<usize>,
}

impl Definition {
    /// Creates a new Definition
    fn new(name: &str, widths: Vec<usize>) -> Self {
        Self {
            name: name.to_string(),
            operand_widths: widths,
        }
    }
}

/// Return the definition based on the Opcode provided
pub fn lookup(op: Opcode) -> anyhow::Result<Definition> {
    match op {
        OP_CONSTANT => Ok(Definition::new("OpConstant", vec![2])),
        OP_ADD => Ok(Definition::new("OpAdd", vec![])),
        OP_POP => Ok(Definition::new("OpPop", vec![])),
        OP_SUB => Ok(Definition::new("OpSub", vec![])),
        OP_MUL => Ok(Definition::new("OpMul", vec![])),
        OP_DIV => Ok(Definition::new("OpDiv", vec![])),
        OP_TRUE => Ok(Definition::new("OpTrue", vec![])),
        OP_FALSE => Ok(Definition::new("OpFalse", vec![])),
        OP_EQUAL => Ok(Definition::new("OpEqual", vec![])),
        OP_NOT_EQUAL => Ok(Definition::new("OpNotEqual", vec![])),
        OP_GREATER_THAN => Ok(Definition::new("OpGreaterThan", vec![])),
        OP_MINUS => Ok(Definition::new("OpMinus", vec![])),
        OP_BANG => Ok(Definition::new("OpBang", vec![])),
        OP_JUMP_NOT_TRUTHY => Ok(Definition::new("OpJumpNotTruthy", vec![2])),
        OP_JUMP => Ok(Definition::new("OpJump", vec![2])),
        OP_NULL => Ok(Definition::new("OpNull", vec![])),
        OP_GET_GLOBAL => Ok(Definition::new("OpGetGlobal", vec![2])), // 65536 global bindings
        OP_SET_GLOBAL => Ok(Definition::new("OpSetGlobal", vec![2])),
        OP_ARRAY => Ok(Definition::new("OpArray", vec![2])),
        OP_HASH => Ok(Definition::new("OpHash", vec![2])),
        OP_INDEX => Ok(Definition::new("OpIndex", vec![])),
        OP_CALL => Ok(Definition::new("OpCall", vec![])),
        OP_RETURN_VALUE => Ok(Definition::new("OpReturnValue", vec![])),
        OP_RETURN => Ok(Definition::new("OpReturn", vec![])),
        OP_GET_LOCAL => Ok(Definition::new("OpGetLocal", vec![1])), // 256 local bindings
        OP_SET_LOCAL => Ok(Definition::new("OpSetLocal", vec![1])),
        OP_ASSIGN_GLOBAL => Ok(Definition::new("OpAssignGlobal", vec![2])),
        _ => Err(anyhow!("opcode must be defined")),
    }
}

/// Creates a single bytecode instruction with the `Opcode` at start,
///
/// following the operands encoded, based on the width specified in the `Opcode` definition.
pub fn make(op: Opcode, operands: &[usize]) -> Instructions {
    let Ok(def) = lookup(op) else {
        return vec![];
    };

    let mut instruction_len = 1; // first byte is for the op_code
    for w in &def.operand_widths {
        instruction_len += w
    }

    let mut instructions = vec![0; instruction_len];
    instructions[0] = op;

    let mut offset = 1;
    for (i, o) in operands.iter().enumerate() {
        let width = def.operand_widths[i];
        match width {
            1 => instructions[offset] = u8::try_from(*o).unwrap(),
            2 => BigEndian::write_u16(&mut instructions[offset..], u16::try_from(*o).unwrap()),
            _ => {}
        };
        offset += width;
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::test_helpers::concat_instructions;
    use test_helpers::*;

    #[test]
    fn test_make() {
        // (op, operands, expected)
        let test_cases = [
            (OP_CONSTANT, vec![65534], vec![OP_CONSTANT, 255, 254]),
            (OP_ADD, vec![], vec![OP_ADD]),
            (OP_GET_LOCAL, vec![254], vec![OP_GET_LOCAL, 254]),
        ];

        for tc in test_cases {
            let instruction = make(tc.0, &tc.1);
            assert_eq!(instruction, tc.2);
        }
    }

    #[test]
    fn test_instructions_string() {
        let instructions = vec![
            make(OP_ADD, &[]),
            make(OP_GET_LOCAL, &[253]),
            make(OP_CONSTANT, &[2]),
            make(OP_CONSTANT, &[65535]),
        ];

        let expected = "0000 OpAdd
0001 OpGetLocal 253
0003 OpConstant 2
0006 OpConstant 65535
";

        let concatted = concat_instructions(instructions);
        assert_eq!(expected, instructions_to_string(&concatted));
    }

    #[test]
    fn test_read_operands() {
        // op, operands, bytes_read
        let test_cases = [(OP_CONSTANT, [65535], 2), (OP_GET_LOCAL, [255], 1)];

        for tc in test_cases {
            let instruction = make(tc.0, &tc.1);
            let def = lookup(tc.0).unwrap();

            let (operands_read, n) = read_operands(&def, &instruction[1..]);
            assert_eq!(n, tc.2);

            for (i, want) in tc.1.into_iter().enumerate() {
                assert_eq!(want, operands_read[i]);
            }
        }
    }
}

#[cfg(test)]
mod test_helpers {
    use super::{helpers, lookup, Definition, Instructions};

    /// Returns a string representation of the instructions
    pub fn instructions_to_string(ins: &Instructions) -> String {
        let mut out = String::new();

        let mut i = 0;
        while i < ins.len() {
            let def = match lookup(ins[i]) {
                Ok(v) => v,
                Err(e) => {
                    out.push_str(format!("ERROR: {e}\n").as_str());
                    continue;
                }
            };

            let (operands, read) = read_operands(&def, &ins[i + 1..]);
            let formatted_instruction = format_instruction(&def, &operands);

            out.push_str(format!("{:04} {}\n", i, formatted_instruction).as_str());
            i += 1 + read;
        }

        out
    }

    /// Decodes operands based on the information provided by the definition and returns
    /// the operands and the number of bytes read.
    pub fn read_operands(def: &Definition, ins: &[u8]) -> (Vec<usize>, usize) {
        let mut operands = vec![0; def.operand_widths.len()];
        let mut offset = 0;

        for (i, width) in def.operand_widths.iter().enumerate() {
            match width {
                1 => operands[i] = helpers::read_u8(&ins[offset..]),
                2 => operands[i] = helpers::read_u16(&ins[offset..]),
                _ => {}
            };
            offset += width;
        }

        (operands, offset)
    }

    /// Return the formatted instruction along with the passed operands.
    ///
    /// Return an error string if the operand count is different from the definition
    pub fn format_instruction(def: &Definition, operands: &Vec<usize>) -> String {
        let operand_count = def.operand_widths.len();
        if operands.len() != operand_count {
            return format!(
                "ERROR: operand len {} does not match defined {}\n",
                operands.len(),
                operand_count
            );
        }

        match operand_count {
            0 => def.name.to_string(),
            1 => format!("{} {}", def.name, operands[0]),
            _ => format!("ERROR: unhandled operandCount for {}\n", def.name),
        }
    }
}
