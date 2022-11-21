use super::Compiler;
use crate::{
    ast::{
        expressions::{self, AllExpressions},
        statements::{self, AllStatements},
        AllNodes,
    },
    code::{OP_ADD, OP_CONSTANT, OP_DIV, OP_MUL, OP_POP, OP_SUB},
    object::{objects::Integer, AllObjects},
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
                AllStatements::Expression(stmt) => self.compile_expression_statement(stmt)?,
                _ => todo!(),
            },
            AllNodes::Expressions(expr) => match expr {
                AllExpressions::IntegerLiteral(v) => self.compile_integer_literal(v)?,
                AllExpressions::InfixExpression(expr) => self.compile_infix_expression(expr)?,
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

    fn compile_infix_expression(&mut self, expr: expressions::InfixExpression) -> Result<()> {
        let Some(left) = expr.left else {
            return Err(anyhow!("infix expression should contain a left expression"));
        };
        self.compile(AllNodes::Expressions(*left))?;

        let Some(right) = expr.right else {
            return Err(anyhow!("infix expression should contain a right expression"));
        };
        self.compile(AllNodes::Expressions(*right))?;

        match expr.operator.as_str() {
            "+" => self.emit(OP_ADD, &[]),
            "-" => self.emit(OP_SUB, &[]),
            "*" => self.emit(OP_MUL, &[]),
            "/" => self.emit(OP_DIV, &[]),
            v => return Err(anyhow!("unknown arithmetic operator: {v}")),
        };
        Ok(())
    }

    fn compile_integer_literal(&mut self, v: expressions::IntegerLiteral) -> Result<()> {
        let integer = AllObjects::Integer(Integer { value: v.value });
        let constant_index = self.add_constant(integer);
        self.emit(OP_CONSTANT, &[constant_index]);
        Ok(())
    }
}
