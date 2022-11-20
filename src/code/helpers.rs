use super::Definition;
use byteorder::ByteOrder;

/// Reads an unsigned 16 bit integer from the buffer and return a general usize.
pub fn read_u16(buf: &[u8]) -> usize {
    byteorder::BigEndian::read_u16(&buf[0..]).into()
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
        1 => return format!("{} {}", def.name, operands[0]),
        _ => format!("ERROR: unhandled operandCount for {}\n", def.name),
    }
}
