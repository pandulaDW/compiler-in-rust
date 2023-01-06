use super::Compiler;
use crate::{
    ast::{
        expressions::{self, AllExpressions},
        statements::{self, AllStatements},
        AllNodes,
    },
    code::{
        make, OP_ADD, OP_ARRAY, OP_ASSIGN_GLOBAL, OP_BANG, OP_CALL, OP_CONSTANT, OP_DIV, OP_EQUAL,
        OP_FALSE, OP_GET_GLOBAL, OP_GET_LOCAL, OP_GREATER_THAN, OP_HASH, OP_INDEX, OP_JUMP,
        OP_JUMP_NOT_TRUTHY, OP_MINUS, OP_MUL, OP_NOT_EQUAL, OP_NULL, OP_POP, OP_RETURN,
        OP_RETURN_VALUE, OP_SET_GLOBAL, OP_SET_LOCAL, OP_SUB, OP_TRUE,
    },
    object::{
        objects::{CompiledFunctionObj, Integer, StringObj},
        AllObjects,
    },
};
use anyhow::{anyhow, Ok, Result};

impl Compiler {
    /// Entrypoint for the compilation process. This method will be called
    /// iteratively by all branches.
    pub fn compile(&mut self, node: AllNodes) -> Result<()> {
        match node {
            AllNodes::Program(p) => {
                for stmt in p.statements {
                    self.compile(AllNodes::Statements(stmt))?;
                }
            }
            AllNodes::Statements(stmt) => match stmt {
                AllStatements::Let(s) => self.compile_let_statements(s)?,
                AllStatements::Block(b) => {
                    for stmt in b.statements {
                        self.compile(AllNodes::Statements(stmt))?;
                    }
                }
                AllStatements::Expression(stmt) => self.compile_expression_statement(stmt)?,
                AllStatements::Return(s) => {
                    self.compile(AllNodes::Expressions(*s.return_value))?;
                    self.emit(OP_RETURN_VALUE, &[]);
                }
                AllStatements::While(_) => unimplemented!(),
            },
            AllNodes::Expressions(expr) => match expr {
                AllExpressions::IntegerLiteral(v) => self.compile_integer_literal(v)?,
                AllExpressions::StringLiteral(v) => self.compile_string_literal(v)?,
                AllExpressions::Boolean(v) => self.compile_boolean_literal(v)?,
                AllExpressions::PrefixExpression(v) => self.compile_prefix_expression(v)?,
                AllExpressions::InfixExpression(v) => self.compile_infix_expression(v)?,
                AllExpressions::IfExpression(v) => self.compile_if_expression(v)?,
                AllExpressions::ArrayLiteral(v) => self.compile_array_literal(v)?,
                AllExpressions::HashLiteral(mut v) => self.compile_hash_literal(&mut v)?,
                AllExpressions::Identifier(v) => self.compile_identifier(v)?,
                AllExpressions::IndexExpression(v) => self.compile_index_expression(v)?,
                AllExpressions::FunctionLiteral(v) => self.compile_function_literals(v)?,
                AllExpressions::CallExpression(v) => self.compile_call_expressions(v)?,
                AllExpressions::Assignment(v) => self.compile_assignment_expression(v)?,
                _ => unimplemented!(),
            },
        }
        Ok(())
    }

    fn compile_expression_statement(
        &mut self,
        stmt: statements::ExpressionStatement,
    ) -> Result<()> {
        let Some(expr) = stmt.expression else {
            return Err(anyhow!("expression statement should contain an expression"));
        };
        self.compile(AllNodes::Expressions(*expr))?;
        self.emit(OP_POP, &[]);
        Ok(())
    }

    fn compile_let_statements(&mut self, s: statements::LetStatement) -> Result<()> {
        self.compile(AllNodes::Expressions(*s.value))?;
        let symbol = self.symbol_table.define(&s.name.value);

        if symbol.is_local() {
            self.emit(OP_SET_LOCAL, &[symbol.index]);
        } else {
            self.emit(OP_SET_GLOBAL, &[symbol.index]);
        }

        Ok(())
    }

    fn compile_identifier(&mut self, v: expressions::Identifier) -> Result<()> {
        let Some(symbol) = self.symbol_table.resolve(&v.value) else {
            return Err(anyhow!("undefined variable {}", &v.value));
        };

        if symbol.is_local() {
            self.emit(OP_GET_LOCAL, &[symbol.index]);
        } else {
            self.emit(OP_GET_GLOBAL, &[symbol.index]);
        }

        Ok(())
    }

    fn compile_infix_expression(&mut self, expr: expressions::InfixExpression) -> Result<()> {
        let Some(left) = expr.left else {
            return Err(anyhow!("infix expression should contain a left expression"));
        };
        let Some(right) = expr.right else {
            return Err(anyhow!("infix expression should contain a right expression"));
        };

        if expr.operator == "<" {
            self.compile(AllNodes::Expressions(*right))?;
            self.compile(AllNodes::Expressions(*left))?;
        } else {
            self.compile(AllNodes::Expressions(*left))?;
            self.compile(AllNodes::Expressions(*right))?;
        }

        match expr.operator.as_str() {
            "+" => self.emit(OP_ADD, &[]),
            "-" => self.emit(OP_SUB, &[]),
            "*" => self.emit(OP_MUL, &[]),
            "/" => self.emit(OP_DIV, &[]),
            ">" | "<" => self.emit(OP_GREATER_THAN, &[]),
            "==" => self.emit(OP_EQUAL, &[]),
            "!=" => self.emit(OP_NOT_EQUAL, &[]),
            v => return Err(anyhow!("unknown arithmetic operator: {v}")),
        };
        Ok(())
    }

    fn compile_prefix_expression(&mut self, expr: expressions::PrefixExpression) -> Result<()> {
        let Some(right) = expr.right else {
            return Err(anyhow!("prefix expression should contain a right expression"));
        };
        self.compile(AllNodes::Expressions(*right))?;

        match expr.operator.as_str() {
            "-" => self.emit(OP_MINUS, &[]),
            "!" => self.emit(OP_BANG, &[]),
            v => return Err(anyhow!("unknown prefix expression: {v}")),
        };

        Ok(())
    }

    fn compile_array_literal(&mut self, expr: expressions::ArrayLiteral) -> Result<()> {
        let n_elements = expr.elements.len();
        for e in expr.elements {
            self.compile(AllNodes::Expressions(e))?;
        }
        self.emit(OP_ARRAY, &[n_elements]);
        Ok(())
    }

    fn compile_hash_literal(&mut self, expr: &mut expressions::HashLiteral) -> Result<()> {
        let n_keys = expr.pairs.len();
        let mut keys: Vec<AllExpressions> = expr.pairs.keys().cloned().collect();
        keys.sort_by_key(|expr| expr.to_string());

        for key in keys {
            let value = expr.pairs.remove(&key).unwrap();
            self.compile(AllNodes::Expressions(key))?;
            self.compile(AllNodes::Expressions(value))?;
        }

        self.emit(OP_HASH, &[n_keys]);
        Ok(())
    }

    fn compile_index_expression(&mut self, expr: expressions::IndexExpression) -> Result<()> {
        self.compile(AllNodes::Expressions(*expr.left))?;
        self.compile(AllNodes::Expressions(*expr.index))?;
        self.emit(OP_INDEX, &[]);
        Ok(())
    }

    fn compile_function_literals(&mut self, expr: expressions::FunctionLiteral) -> Result<()> {
        self.enter_scope();

        for param in expr.parameters {
            self.symbol_table.define(&param.value);
        }

        self.compile(AllNodes::Statements(AllStatements::Block(expr.body)))?;

        // replace last pop with return
        if self.last_instruction_is(OP_POP) {
            let last_pos = self.scopes[self.scope_index].last_instruction.position;
            self.replace_instruction(last_pos, make(OP_RETURN_VALUE, &[]));
            self.scopes[self.scope_index].last_instruction.opcode = OP_RETURN_VALUE;
        }

        // empty function case
        if !self.last_instruction_is(OP_RETURN_VALUE) {
            self.emit(OP_RETURN, &[]);
        }

        let fn_instructions = self.leave_scope();

        let compiled_fn = AllObjects::CompiledFunction(CompiledFunctionObj::new(fn_instructions));
        let constant_index = self.add_constant(compiled_fn);
        self.emit(OP_CONSTANT, &[constant_index]);

        Ok(())
    }

    fn compile_if_expression(&mut self, expr: expressions::IfExpression) -> Result<()> {
        self.compile(AllNodes::Expressions(*expr.condition))?;

        // Emit an `OP_JUMP_NOT_TRUTHY` with a bogus value
        let jump_not_truthy_position = self.emit(OP_JUMP_NOT_TRUTHY, &[9999]);

        self.compile(AllNodes::Statements(AllStatements::Block(expr.consequence)))?;
        if self.last_instruction_is(OP_POP) {
            self.remove_last_pop();
        }

        // Emit an `OP_JUMP` with a bogus value
        let jump_position = self.emit(OP_JUMP, &[9999]);

        let after_consequence_pos = self.current_instructions().len();
        self.change_operand(jump_not_truthy_position, after_consequence_pos);

        if expr.alternative.is_none() {
            self.emit(OP_NULL, &[]);
        } else {
            self.compile(AllNodes::Statements(AllStatements::Block(
                expr.alternative.unwrap(),
            )))?;

            if self.last_instruction_is(OP_POP) {
                self.remove_last_pop();
            }
        }

        let after_alternative_pos = self.current_instructions().len();
        self.change_operand(jump_position, after_alternative_pos);

        Ok(())
    }

    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = self.current_instructions()[op_pos];
        let new_instruction = make(op, &[operand]);
        self.replace_instruction(op_pos, new_instruction);
    }

    fn compile_integer_literal(&mut self, v: expressions::IntegerLiteral) -> Result<()> {
        let integer = AllObjects::Integer(Integer { value: v.value });
        let constant_index = self.add_constant(integer);
        self.emit(OP_CONSTANT, &[constant_index]);
        Ok(())
    }

    fn compile_string_literal(&mut self, v: expressions::StringLiteral) -> Result<()> {
        let string_obj = AllObjects::StringObj(StringObj::new(&v.token.literal));
        let constant_index = self.add_constant(string_obj);
        self.emit(OP_CONSTANT, &[constant_index]);
        Ok(())
    }

    fn compile_boolean_literal(&mut self, v: expressions::Boolean) -> Result<()> {
        match v.value {
            true => self.emit(OP_TRUE, &[]),
            false => self.emit(OP_FALSE, &[]),
        };
        Ok(())
    }

    fn compile_call_expressions(&mut self, v: expressions::CallExpression) -> Result<()> {
        self.compile(AllNodes::Expressions(*v.function))?;

        let num_args = v.arguments.len();
        for arg in v.arguments {
            self.compile(AllNodes::Expressions(arg))?;
        }

        self.emit(OP_CALL, &[num_args]);
        Ok(())
    }

    fn compile_assignment_expression(
        &mut self,
        v: expressions::AssignmentExpression,
    ) -> Result<()> {
        self.compile(AllNodes::Expressions(*v.value))?;

        let Some(resolved )= self.symbol_table.resolve(&v.ident.value) else {
             return Err(anyhow!("variable with name {}, not found",&v.ident.value));
        };

        self.emit(OP_ASSIGN_GLOBAL, &[resolved.index]);

        Ok(())
    }
}
