use std::collections::HashMap;

use super::{frame::Frame, FALSE, NULL, TRUE, VM};
use crate::{
    code::{self, *},
    object::{
        builtins::get_builtin_function,
        objects::{ArrayObj, Closure, HashMapObj, Integer, StringObj},
        AllObjects, Object, ObjectType,
    },
};
use anyhow::{anyhow, Result};

impl VM {
    /// Runs the bytecode instructions from start to finish.
    pub fn run(&mut self) -> Result<()> {
        while self.current_frame().ip < self.current_frame().instructions().len() {
            let ip = self.current_frame().ip;
            let op = self.current_frame().instructions()[ip];

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
                OP_SET_LOCAL => self.run_set_local_instruction()?,
                OP_GET_LOCAL => self.run_get_local_instruction()?,
                OP_ARRAY => self.run_array_literal_instruction()?,
                OP_HASH => self.run_hash_literal_instruction()?,
                OP_INDEX => self.run_index_expression()?,
                OP_CLOSURE => self.run_closure_instruction()?,
                OP_CALL => self.run_call_expression()?,
                OP_ASSIGN_GLOBAL => self.run_assign_global_instruction()?,
                OP_GET_BUILTIN => self.run_get_builtin()?,
                OP_RETURN_VALUE => {
                    self.pop_frame();
                }
                OP_RETURN => {
                    self.pop_frame();
                    self.push(NULL)?;
                }
                OP_POP => {
                    self.pop()?;
                }
                OP_JUMP_NOT_TRUTHY => self.run_jump_not_truthy_instruction()?,
                OP_JUMP => self.run_jump_instruction()?,
                OP_NULL => self.push(NULL)?,
                _ => {}
            }
            self.current_frame().ip += 1;
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
            "arithmetic operations are only supported between strings or integers"
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
        let ip = self.current_frame().ip;
        let const_index = code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        if self.constants.get(const_index).is_none() {
            return Err(anyhow!("constant at the index {const_index} not found"));
        }
        self.push(self.constants[const_index].clone())?;
        self.current_frame().ip += 2;
        Ok(())
    }

    fn run_set_global_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let global_index =
            code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 2;

        let last_pushed = self.pop()?;
        if self.globals.get(global_index).is_none() {
            self.globals.push(last_pushed);
        } else {
            self.globals[global_index] = last_pushed;
        }

        Ok(())
    }

    fn run_assign_global_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let var_index = code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 2;

        let last_pushed = self.pop()?;
        if self.globals.get(var_index).is_none() {
            return Err(anyhow!("variable at index {var_index} not found"));
        } else {
            self.globals[var_index] = last_pushed;
        }

        self.push(NULL)?; // assignment is an expression and will return null
        Ok(())
    }

    fn run_set_local_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let local_index = code::helpers::read_u8(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 1;

        let last_pushed = self.pop()?;
        if self.current_frame().locals.get(local_index).is_none() {
            self.current_frame().locals.push(last_pushed);
        } else {
            self.current_frame().locals[local_index] = last_pushed;
        }

        Ok(())
    }

    fn run_get_builtin(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let builtin_index =
            code::helpers::read_u8(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 1;

        let Some(func) = get_builtin_function(builtin_index) else {
            return Err(anyhow!("builtin function with index {builtin_index} not found"));
        };

        self.push(func)?;
        Ok(())
    }

    fn run_get_global_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let global_index =
            code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 2;
        let Some(v) = self.globals.get(global_index) else {
            return Err(anyhow!("variable at index {global_index} not found"));
        };
        self.push(v.clone())?;
        Ok(())
    }

    fn run_get_local_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let local_index = code::helpers::read_u8(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 1;
        let Some(v) = self.current_frame().locals.get(local_index) else {
            return Err(anyhow!("variable at index {local_index} not found"));
        };
        let cloned = v.clone();
        self.push(cloned)?;
        Ok(())
    }

    fn run_array_literal_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let arr_len = code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 2;
        let mut elements = Vec::with_capacity(arr_len);

        for _ in 0..arr_len {
            let element = self.pop()?;
            elements.push(element);
        }
        elements.reverse();

        self.push(AllObjects::ArrayObj(ArrayObj::new(elements)))?;
        Ok(())
    }

    fn run_hash_literal_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let map_len = code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 2;
        let mut map = HashMap::new();

        for _ in 0..map_len {
            let value = self.pop()?;
            let key = self.pop()?;
            map.insert(key, value);
        }

        self.push(AllObjects::HashMap(HashMapObj::new(map)))?;
        Ok(())
    }

    fn run_index_expression(&mut self) -> Result<()> {
        let index = self.pop()?;
        let indexable = self.pop()?;

        if indexable.object_type() == ObjectType::Array {
            let index = match index {
                AllObjects::Integer(v) => v,
                _ => return Err(anyhow!("index should be an integer")),
            };
            let index_usize: usize = match index.value.try_into() {
                Ok(v) => v,
                Err(_) => return Err(anyhow!("index should be a positive integer")),
            };

            let arr = match indexable {
                AllObjects::ArrayObj(v) => v,
                _ => unreachable!(),
            };
            let borrowed = arr.elements.borrow();
            let Some(value) = borrowed.get(index_usize) else {
                return Err(anyhow!("index out of bounds"));
            };
            self.push(value.clone())?;
            return Ok(());
        }

        if indexable.object_type() == ObjectType::HashMap {
            let map_obj = match indexable {
                AllObjects::HashMap(v) => v,
                _ => unreachable!(),
            };
            let borrowed = map_obj.map.borrow();
            let value = match borrowed.get(&index) {
                Some(v) => v.clone(),
                None => NULL,
            };
            self.push(value)?;
            return Ok(());
        }

        Err(anyhow!(
            "indexing is only supported for arrays and hash-maps"
        ))
    }

    fn run_jump_not_truthy_instruction(&mut self) -> Result<()> {
        let condition = match Self::cast_obj_to_bool(self.pop()?) {
            AllObjects::Boolean(v) => v,
            _ => unreachable!(),
        };

        if !condition.value {
            let ip = self.current_frame().ip;
            let jump_position =
                code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
            self.current_frame().ip = jump_position - 1; // since ip gets incremented at the end of the loop
            return Ok(());
        }

        // consume the jump instruction
        self.current_frame().ip += 2;

        Ok(())
    }

    fn run_jump_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let jump_position =
            code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip = jump_position - 1; // since ip gets incremented at in the loop
        Ok(())
    }

    fn run_closure_instruction(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let const_index = code::helpers::read_u16(&self.current_frame().instructions()[(ip + 1)..]);
        _ = code::helpers::read_u8(&self.current_frame().instructions()[(ip + 3)..]);
        self.current_frame().ip += 3;

        let func = match self.constants.get(const_index) {
            Some(obj) => match obj {
                AllObjects::CompiledFunction(v) => v,
                v => return Err(anyhow!("not a function: {}", v.inspect())),
            },
            None => return Err(anyhow!("constant at index {const_index} not found")),
        };

        let closure = Closure::new(func.to_owned());
        self.push(AllObjects::Closure(closure))?;
        Ok(())
    }

    fn run_call_expression(&mut self) -> Result<()> {
        let ip = self.current_frame().ip;
        let num_args = code::helpers::read_u8(&self.current_frame().instructions()[(ip + 1)..]);
        self.current_frame().ip += 1;

        let mut local_args = (0..num_args)
            .filter_map(|_| self.pop().ok())
            .collect::<Vec<AllObjects>>();
        local_args.reverse();

        match self.pop()? {
            AllObjects::Closure(c) => {
                if local_args.len() != c.func.num_args {
                    return Err(anyhow!(
                        "wrong number of arguments: want={}, got={}",
                        c.func.num_args,
                        local_args.len()
                    ));
                }
                self.push_frame(Frame::new(c.func, local_args));
                self.run()?;
            }
            AllObjects::BuiltinFunction(builtin) => {
                if local_args.len() != builtin.num_params && builtin.num_params != usize::MAX {
                    return Err(anyhow!(
                        "wrong number of arguments: want={}, got={}",
                        builtin.num_params,
                        local_args.len()
                    ));
                }
                let result = (builtin.func)(local_args)?;
                self.push(result)?;
            }
            v => return Err(anyhow!("expected a function, found {}", v.inspect())),
        };

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
