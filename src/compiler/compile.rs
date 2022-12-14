use super::Compiler;
use crate::{
    ast::{
        expressions::{self, AllExpressions},
        statements::{self, AllStatements},
        AllNodes,
    },
    code::{
        make, OP_ADD, OP_BANG, OP_CONSTANT, OP_DIV, OP_EQUAL, OP_FALSE, OP_GET_GLOBAL,
        OP_GREATER_THAN, OP_JUMP, OP_JUMP_NOT_TRUTHY, OP_MINUS, OP_MUL, OP_NOT_EQUAL, OP_NULL,
        OP_POP, OP_SET_GLOBAL, OP_SUB, OP_TRUE,
    },
    object::{
        objects::{Integer, StringObj},
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
                AllStatements::Let(s) => {
                    self.compile(AllNodes::Expressions(*s.value))?;
                    let symbol = self.symbol_table.define(&s.name.value);
                    self.emit(OP_SET_GLOBAL, &[symbol.index]);
                }
                AllStatements::Block(b) => {
                    for stmt in b.statements {
                        self.compile(AllNodes::Statements(stmt))?;
                    }
                }
                AllStatements::Expression(stmt) => self.compile_expression_statement(stmt)?,
                _ => todo!(),
            },
            AllNodes::Expressions(expr) => match expr {
                AllExpressions::IntegerLiteral(v) => self.compile_integer_literal(v)?,
                AllExpressions::StringLiteral(v) => self.compile_string_literal(v)?,
                AllExpressions::Boolean(v) => self.compile_boolean_literal(v)?,
                AllExpressions::PrefixExpression(v) => self.compile_prefix_expression(v)?,
                AllExpressions::InfixExpression(v) => self.compile_infix_expression(v)?,
                AllExpressions::IfExpression(v) => self.compile_if_expression(v)?,
                AllExpressions::Identifier(v) => self.compile_identifier(v)?,
                _ => todo!(),
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

    fn compile_identifier(&mut self, v: expressions::Identifier) -> Result<()> {
        let Some(symbol) = self.symbol_table.resolve(&v.value) else {
            return Err(anyhow!("undefined variable {}", &v.value));
        };
        self.emit(OP_GET_GLOBAL, &[symbol.index]);
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

    fn compile_if_expression(&mut self, expr: expressions::IfExpression) -> Result<()> {
        self.compile(AllNodes::Expressions(*expr.condition))?;

        // Emit an `OP_JUMP_NOT_TRUTHY` with a bogus value
        let jump_not_truthy_position = self.emit(OP_JUMP_NOT_TRUTHY, &[9999]);

        self.compile(AllNodes::Statements(AllStatements::Block(expr.consequence)))?;
        if self.last_instruction.opcode == OP_POP {
            self.remove_last_pop();
        }

        // Emit an `OP_JUMP` with a bogus value
        let jump_position = self.emit(OP_JUMP, &[9999]);

        let after_consequence_pos = self.instructions.len();
        self.change_operand(jump_not_truthy_position, after_consequence_pos);

        if expr.alternative.is_none() {
            self.emit(OP_NULL, &[]);
        } else {
            self.compile(AllNodes::Statements(AllStatements::Block(
                expr.alternative.unwrap(),
            )))?;

            if self.last_instruction.opcode == OP_POP {
                self.remove_last_pop();
            }
        }

        let after_alternative_pos = self.instructions.len();
        self.change_operand(jump_position, after_alternative_pos);

        Ok(())
    }

    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = self.instructions[op_pos];
        let new_instruction = make(op, &[operand]);

        // replace the instructions bytes with the new instruction
        self.instructions[op_pos..(op_pos + new_instruction.len())]
            .copy_from_slice(&new_instruction);
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
}
