use super::Compiler;
use crate::{
    ast::{
        expressions::{self, AllExpressions},
        statements::AllStatements,
        AllNodes,
    },
    code::{OP_ADD, OP_CONSTANT},
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
                AllStatements::Expression(expr_stmt) => {
                    let Some(expr) = expr_stmt.expression else {
                        return Err(anyhow!("expression statement should contain an expression"));
                    };
                    return self.compile(AllNodes::Expressions(*expr));
                }
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
