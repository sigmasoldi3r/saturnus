use ast::*;

use crate::parser::{ast, Script};

#[derive(Debug)]
pub struct VisitError;

pub trait Visitor<T> {
    // Those need to be implemented explicitly by the user:
    fn visit_return(&self, ctx: T, stmt: &Return) -> Result<T, VisitError>;
    fn visit_class(&self, ctx: T, stmt: &Class) -> Result<T, VisitError>;
    fn visit_fn(&self, ctx: T, stmt: &Function) -> Result<T, VisitError>;
    fn visit_assignment(&self, ctx: T, stmt: &Assignment) -> Result<T, VisitError>;
    fn visit_declaration(&self, ctx: T, stmt: &Let) -> Result<T, VisitError>;
    fn visit_expression_statement(&self, ctx: T, stmt: &Expression) -> Result<T, VisitError>;
    fn visit_lambda(&self, ctx: T, expr: &Lambda) -> Result<T, VisitError>;
    fn visit_reference(&self, ctx: T, expr: &MemberExpression) -> Result<T, VisitError>;
    fn visit_call(&self, ctx: T, expr: &CallExpression) -> Result<T, VisitError>;
    fn visit_tuple(&self, ctx: T, expr: &Tuple) -> Result<T, VisitError>;
    fn visit_number(&self, ctx: T, expr: &Number) -> Result<T, VisitError>;
    fn visit_string(&self, ctx: T, expr: &StringLiteral) -> Result<T, VisitError>;
    fn visit_unit(&self, ctx: T) -> Result<T, VisitError>;
    fn visit_binary(&self, ctx: T, expr: &BinaryExpression) -> Result<T, VisitError>;
    fn visit_unary(&self, ctx: T, expr: &UnaryExpression) -> Result<T, VisitError>;
    fn visit_if(&self, ctx: T, expr: &If) -> Result<T, VisitError>;
    fn visit_table(&self, ctx: T, expr: &Table) -> Result<T, VisitError>;
    fn visit_vector(&self, ctx: T, expr: &Vector) -> Result<T, VisitError>;
    fn visit_for(&self, ctx: T, expr: &For) -> Result<T, VisitError>;
    fn visit_while(&self, ctx: T, expr: &While) -> Result<T, VisitError>;
    fn visit_loop(&self, ctx: T, expr: &Loop) -> Result<T, VisitError>;
    fn visit_match(&self, ctx: T, expr: &Match) -> Result<T, VisitError>;
    fn visit_1tuple(&self, ctx: T, expr: &Expression) -> Result<T, VisitError>;
    fn visit_identifier(&self, ctx: T, expr: &Identifier) -> Result<T, VisitError>;
    fn visit_do(&self, ctx: T, expr: &Do) -> Result<T, VisitError>;
    fn enter_script(&self, ctx: T, _script: &Script) -> Result<T, VisitError> {
        Ok(ctx)
    }
    fn exit_script(&self, ctx: T, _script: &Script) -> Result<T, VisitError> {
        Ok(ctx)
    }

    // Generically implementable matching patterns:
    fn visit_expression(&self, ctx: T, expression: &Expression) -> Result<T, VisitError> {
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
            Expression::Table(e) => self.visit_table(ctx, e),
            Expression::Vector(e) => self.visit_vector(ctx, e),
            Expression::Tuple1(e) => self.visit_1tuple(ctx, e),
            Expression::Identifier(e) => self.visit_identifier(ctx, e),
            Expression::Do(e) => self.visit_do(ctx, e),
        }
    }
    fn visit_script(&self, ctx: T, script: &Script) -> Result<T, VisitError> {
        let ctx = self.enter_script(ctx, script)?;
        let ctx = script
            .statements
            .iter()
            .fold(Ok(ctx), |ctx, stmt| match stmt {
                ast::Statement::If(e) => self.visit_if(ctx?, e),
                ast::Statement::For(e) => self.visit_for(ctx?, e),
                ast::Statement::Loop(e) => self.visit_loop(ctx?, e),
                ast::Statement::While(e) => self.visit_while(ctx?, e),
                ast::Statement::Return(e) => self.visit_return(ctx?, e),
                ast::Statement::Class(e) => self.visit_class(ctx?, e),
                ast::Statement::Function(e) => self.visit_fn(ctx?, e),
                ast::Statement::Assignment(e) => self.visit_assignment(ctx?, e),
                ast::Statement::Let(e) => self.visit_declaration(ctx?, e),
                ast::Statement::Match(e) => self.visit_match(ctx?, e),
                ast::Statement::Expression(e) => self.visit_expression_statement(ctx?, e),
            })?;
        self.exit_script(ctx, script)
    }
}
