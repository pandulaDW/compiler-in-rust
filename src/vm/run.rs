use super::{FALSE, TRUE, VM};
use crate::{
    code::{self, *},
    object::{objects::Integer, AllObjects, Object},
};
use anyhow::{anyhow, Result};

impl VM {
    /// Runs the bytecode instructions from start to finish.
    pub fn run(&mut self) -> Result<()> {
        while self.ip < self.instructions.len() {
            let op = self.instructions[self.ip];
            match op {
                OP_CONSTANT => self.run_constant_instruction()?,
                OP_ADD | OP_SUB | OP_MUL | OP_DIV => self.run_arithmetic_operations(op)?,
                OP_EQUAL | OP_NOT_EQUAL | OP_GREATER_THAN => self.run_boolean_operations(op)?,
                OP_TRUE => self.push(TRUE)?,
                OP_FALSE => self.push(FALSE)?,
                OP_MINUS => self.run_prefix_minus()?,
                OP_BANG => self.run_prefix_bang()?,
                OP_POP => {
                    self.pop()?;
                }
                _ => {}
            }
            self.ip += 1;
        }

        Ok(())
    }

    fn run_arithmetic_operations(&mut self, op: Opcode) -> Result<()> {
        let right_value = match self.pop()? {
            AllObjects::Integer(v) => v,
            v => return Err(anyhow!("expected an INTEGER, found {}", v.inspect())),
        };
        let left_value = match self.pop()? {
            AllObjects::Integer(v) => v,
            v => return Err(anyhow!("expected an INTEGER, FOUND {}", v.inspect())),
        };
        let result = match op {
            OP_ADD => left_value.value + right_value.value,
            OP_SUB => left_value.value - right_value.value,
            OP_MUL => left_value.value * right_value.value,
            OP_DIV => left_value.value / right_value.value,
            _ => unreachable!(),
        };

        self.push(AllObjects::Integer(Integer { value: result }))
    }

    fn run_boolean_operations(&mut self, op: Opcode) -> Result<()> {
        let right = self.pop()?;
        let left = self.pop()?;

        if left.is_integer() && right.is_integer() {
            return self.run_comparison_for_ints(op, left, right);
        }
        if left.is_boolean() && right.is_boolean() {
            return self.run_comparison_for_bools(op, left, right);
        }
        Err(anyhow!(
            "left {} and right {} operand types doesn't match",
            left.object_type(),
            right.object_type()
        ))
    }

    fn run_constant_instruction(&mut self) -> Result<()> {
        let const_index = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
        if self.constants.get(const_index).is_none() {
            return Err(anyhow!("constant at the index {const_index} not found"));
        }
        self.push(self.constants[const_index].clone())?;
        self.ip += 2;
        Ok(())
    }

    fn run_comparison_for_ints(&mut self, op: Opcode, l: AllObjects, r: AllObjects) -> Result<()> {
        let left = match l {
            AllObjects::Integer(v) => v,
            _ => unreachable!(),
        };
        let right = match r {
            AllObjects::Integer(v) => v,
            _ => unreachable!(),
        };
        let result = match op {
            OP_EQUAL => left.value == right.value,
            OP_NOT_EQUAL => left.value != right.value,
            OP_GREATER_THAN => left.value > right.value,
            _ => unreachable!(),
        };
        self.push(Self::get_bool_constant(result))
    }

    fn run_comparison_for_bools(&mut self, op: Opcode, l: AllObjects, r: AllObjects) -> Result<()> {
        let left = match l {
            AllObjects::Boolean(v) => v,
            _ => unreachable!(),
        };
        let right = match r {
            AllObjects::Boolean(v) => v,
            _ => unreachable!(),
        };
        let result = match op {
            OP_EQUAL => left.value == right.value,
            OP_NOT_EQUAL => left.value != right.value,
            OP_GREATER_THAN => left.value & !right.value,
            _ => unreachable!(),
        };
        self.push(Self::get_bool_constant(result))
    }

    fn run_prefix_minus(&mut self) -> Result<()> {
        let right = match self.pop()? {
            AllObjects::Integer(v) => v,
            v => return Err(anyhow!("expected an INTEGER, found {}", v.inspect())),
        };
        self.push(AllObjects::Integer(Integer {
            value: -right.value,
        }))
    }

    fn run_prefix_bang(&mut self) -> Result<()> {
        let result = match self.pop()? {
            TRUE => FALSE,
            FALSE => TRUE,
            _ => FALSE,
        };
        self.push(result)
    }

    fn get_bool_constant(val: bool) -> AllObjects {
        if val {
            return TRUE;
        }
        FALSE
    }
}
