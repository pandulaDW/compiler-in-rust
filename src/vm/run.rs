use super::{FALSE, NULL, TRUE, VM};
use crate::{
    code::{self, *},
    object::{
        objects::{ArrayObj, Integer, StringObj},
        AllObjects, Object,
    },
};
use anyhow::{anyhow, Ok, Result};

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
                OP_SET_GLOBAL => self.run_set_global_instruction()?,
                OP_GET_GLOBAL => self.run_get_global_instruction()?,
                OP_ARRAY => self.run_array_literal_instruction()?,
                OP_POP => {
                    self.pop()?;
                }
                OP_JUMP_NOT_TRUTHY => self.run_jump_not_truthy_instruction()?,
                OP_JUMP => self.run_jump_instruction()?,
                OP_NULL => self.push(NULL)?,
                _ => {}
            }
            self.ip += 1;
        }

        Ok(())
    }

    fn run_arithmetic_operations(&mut self, op: Opcode) -> Result<()> {
        let right = self.pop()?;
        let left = self.pop()?;

        if left.is_string() && right.is_string() {
            if op != OP_ADD {
                return Err(anyhow!("incorrect operation on strings"));
            }

            let right_val = match right {
                AllObjects::StringObj(v) => v,
                _ => unreachable!(),
            };
            let left_val = match left {
                AllObjects::StringObj(v) => v,
                _ => unreachable!(),
            };
            let concatenated = format!("{}{}", left_val.value, right_val.value);
            return self.push(AllObjects::StringObj(StringObj::new(&concatenated)));
        }

        if left.is_integer() && right.is_integer() {
            let right_value = match right {
                AllObjects::Integer(v) => v,
                _ => unreachable!(),
            };
            let left_value = match left {
                AllObjects::Integer(v) => v,
                _ => unreachable!(),
            };
            let result = match op {
                OP_ADD => left_value.value + right_value.value,
                OP_SUB => left_value.value - right_value.value,
                OP_MUL => left_value.value * right_value.value,
                OP_DIV => left_value.value / right_value.value,
                _ => unreachable!(),
            };

            return self.push(AllObjects::Integer(Integer { value: result }));
        }

        Err(anyhow!(
            "addition is only supported between strings or integers"
        ))
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

    fn run_set_global_instruction(&mut self) -> Result<()> {
        let global_index = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
        self.ip += 2;

        let last_pushed = self.pop()?;
        if self.globals.get(global_index).is_none() {
            self.globals.push(last_pushed);
        } else {
            self.globals[global_index] = last_pushed;
        }

        Ok(())
    }

    fn run_get_global_instruction(&mut self) -> Result<()> {
        let global_index = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
        self.ip += 2;
        let Some(v) = self.globals.get(global_index) else {
            return Err(anyhow!("variable at index {global_index} not found"));
        };
        self.push(v.clone())?;
        Ok(())
    }

    fn run_array_literal_instruction(&mut self) -> Result<()> {
        let arr_len = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
        self.ip += 2;
        let mut elements = Vec::with_capacity(arr_len);

        for _ in 0..arr_len {
            let element = self.pop()?;
            elements.push(element);
        }
        elements.reverse();

        self.push(AllObjects::ArrayObj(ArrayObj::new(elements)))?;
        Ok(())
    }

    fn run_jump_not_truthy_instruction(&mut self) -> Result<()> {
        let condition = match Self::cast_obj_to_bool(self.pop()?) {
            AllObjects::Boolean(v) => v,
            _ => unreachable!(),
        };

        if !condition.value {
            let jump_position = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
            self.ip = jump_position - 1; // since ip gets incremented at the end of the loop
            return Ok(());
        }

        // consume the jump instruction
        self.ip += 2;

        Ok(())
    }

    fn run_jump_instruction(&mut self) -> Result<()> {
        let jump_position = code::helpers::read_u16(&self.instructions[(self.ip + 1)..]);
        self.ip = jump_position - 1; // since ip gets incremented at in the loop
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
            NULL => TRUE,
            _ => FALSE,
        };
        self.push(result)
    }

    fn cast_obj_to_bool(obj: AllObjects) -> AllObjects {
        match obj {
            FALSE | NULL => FALSE,
            _ => TRUE,
        }
    }

    fn get_bool_constant(val: bool) -> AllObjects {
        if val {
            return TRUE;
        }
        FALSE
    }
}
