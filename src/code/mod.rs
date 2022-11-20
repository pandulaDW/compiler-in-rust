#![allow(dead_code)]

pub mod helpers;

use anyhow::anyhow;
use byteorder::{BigEndian, ByteOrder};
use iota::iota;

/// Opcode is an alias to a byte
pub type Opcode = u8;

/// Instructions are a vector of u8s, which contains all the information needed
/// to carry out an instruction.
pub type Instructions = Vec<Opcode>;

// List of OpCode constants which has a width of u8
iota! {
    pub const OP_CONSTANT: Opcode = 1 << iota;
    , B
}

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
            2 => BigEndian::write_u16(&mut instructions[offset..], u16::try_from(*o).unwrap()),
            _ => {}
        };
        offset += width;
    }

    instructions
}

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
        let formatted_instruction = helpers::format_instruction(&def, &operands);

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
            2 => operands[i] = helpers::read_u16(&ins[offset..]),
            _ => {}
        };
        offset += width;
    }

    (operands, offset)
}

#[cfg(test)]
mod tests {
    use super::{make, OP_CONSTANT};

    #[test]
    fn test_make() {
        // (op, operands, expected)
        let test_cases = [(OP_CONSTANT, [65534], vec![OP_CONSTANT, 255, 254])];

        for tc in test_cases {
            let instruction = make(tc.0, &tc.1);
            assert_eq!(instruction, tc.2);
        }
    }
}
