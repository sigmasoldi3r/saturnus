use std::error::Error;

use ast::*;

use crate::parser::{ast, Script};

use super::builder::Builder;

#[derive(Debug)]
pub struct VisitError(pub Box<dyn Error>);

pub type Result = std::result::Result<Builder, VisitError>;

pub trait Visitor {
    // Expressions
    fn visit_reference(&self, ctx: Builder, expr: &MemberExpression) -> Result;
    fn visit_call(&self, ctx: Builder, expr: &CallExpression) -> Result;
    fn visit_binary(&self, ctx: Builder, expr: &BinaryExpression) -> Result;
    fn visit_unary(&self, ctx: Builder, expr: &UnaryExpression) -> Result;
    fn visit_spread(&self, ctx: Builder, expr: &SpreadExpression) -> Result;
    fn visit_wrapped_expression(&self, ctx: Builder, expr: &Expression) -> Result;
    fn visit_identifier(&self, ctx: Builder, expr: &Identifier) -> Result;

    // Literals
    fn visit_lambda(&self, ctx: Builder, expr: &Lambda) -> Result;
    fn visit_tuple(&self, ctx: Builder, expr: &Tuple) -> Result;
    fn visit_unit(&self, ctx: Builder) -> Result;
    fn visit_number(&self, ctx: Builder, expr: &Number) -> Result;
    fn visit_string(&self, ctx: Builder, expr: &StringLiteral) -> Result;
    fn visit_table(&self, ctx: Builder, expr: &Table) -> Result;
    fn visit_vector(&self, ctx: Builder, expr: &Vector) -> Result;

    // Statements
    fn visit_return(&self, ctx: Builder, stmt: &Return) -> Result;
    fn visit_class(&self, ctx: Builder, stmt: &Class) -> Result;
    fn visit_fn(&self, ctx: Builder, stmt: &Function) -> Result;
    fn visit_assignment(&self, ctx: Builder, stmt: &Assignment) -> Result;
    fn visit_declaration(&self, ctx: Builder, stmt: &Let) -> Result;
    fn visit_expression_statement(&self, ctx: Builder, stmt: &Expression) -> Result;
    fn visit_use_statement(&self, ctx: Builder, stmt: &UseStatement) -> Result;
    fn visit_extern_block(&self, ctx: Builder, stmt: &Extern) -> Result;

    // Looping
    fn visit_for(&self, ctx: Builder, expr: &For) -> Result;
    fn visit_while(&self, ctx: Builder, expr: &While) -> Result;
    fn visit_loop(&self, ctx: Builder, expr: &Loop) -> Result;

    // Conditionals
    fn visit_if(&self, ctx: Builder, expr: &If) -> Result;
    fn visit_match(&self, ctx: Builder, expr: &Match) -> Result;

    fn visit_block_expression(&self, ctx: Builder, expr: &Do) -> Result;
    fn visit_script(&self, ctx: Builder, script: &Script) -> Result;

    // Macros
    fn visit_macro_decorator(&self, ctx: Builder, stmt: &MacroDecorator) -> Result;
    fn visit_macro_call(&self, ctx: Builder, expr: &MacroCallExpression) -> Result;

    // Generically implementable matching patterns:
    fn visit_expression(&self, ctx: Builder, expression: &Expression) -> Result {
        match expression {
            Expression::Lambda(e) => self.visit_lambda(ctx, e),
            Expression::Reference(e) => self.visit_reference(ctx, e),
            Expression::Call(e) => self.visit_call(ctx, e),
            Expression::Tuple(e) => self.visit_tuple(ctx, e),
            Expression::Number(e) => self.visit_number(ctx, e),
            Expression::String(e) => self.visit_string(ctx, e),
            Expression::Unit => self.visit_unit(ctx),
            Expression::Binary(e) => self.visit_binary(ctx, e),
            Expression::Unary(e) => self.visit_unary(ctx, e),
            Expression::Spread(e) => self.visit_spread(ctx, e),
            Expression::Table(e) => self.visit_table(ctx, e),
            Expression::Vector(e) => self.visit_vector(ctx, e),
            Expression::Tuple1(e) => self.visit_wrapped_expression(ctx, e),
            Expression::Identifier(e) => self.visit_identifier(ctx, e),
            Expression::Do(e) => self.visit_block_expression(ctx, e),
            Expression::MacroCall(e) => self.visit_macro_call(ctx, e),
        }
    }
    fn visit_statement(&self, ctx: Builder, statement: &Statement) -> Result {
        match statement {
            ast::Statement::MacroDecorator(e) => self.visit_macro_decorator(ctx, e),
            ast::Statement::If(e) => self.visit_if(ctx, e),
            ast::Statement::For(e) => self.visit_for(ctx, e),
            ast::Statement::Loop(e) => self.visit_loop(ctx, e),
            ast::Statement::While(e) => self.visit_while(ctx, e),
            ast::Statement::Return(e) => self.visit_return(ctx, e),
            ast::Statement::Class(e) => self.visit_class(ctx, e),
            ast::Statement::Function(e) => self.visit_fn(ctx, e),
            ast::Statement::Assignment(e) => self.visit_assignment(ctx, e),
            ast::Statement::Let(e) => self.visit_declaration(ctx, e),
            ast::Statement::Match(e) => self.visit_match(ctx, e),
            ast::Statement::Expression(e) => self.visit_expression_statement(ctx, e),
            ast::Statement::UseStatement(e) => self.visit_use_statement(ctx, e),
            ast::Statement::Extern(e) => self.visit_extern_block(ctx, e),
        }
    }
    fn visit_block(&self, ctx: Builder, script: &Script) -> Result {
        script
            .statements
            .iter()
            .fold(Ok(ctx), |ctx, stmt| self.visit_statement(ctx?, stmt))
    }
}
