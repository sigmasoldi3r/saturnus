use crate::parser::ast::*;

use super::{Builder, VisitError, Visitor};

struct ConstructionError;

pub trait CodeEmitter {
    fn build_class(&self, target: &Class) -> Result<Builder, ConstructionError>;
}

pub struct EmitterVisitor(Box<dyn CodeEmitter>);

impl Visitor<Builder> for EmitterVisitor {
    fn visit_return(&self, ctx: Builder, stmt: &Return) -> Result<Builder, VisitError> {}

    fn visit_class(&self, ctx: Builder, stmt: &Class) -> Result<Builder, VisitError> {
        // stmt.
    }

    fn visit_fn(&self, ctx: Builder, stmt: &Function) -> Result<Builder, VisitError> {}

    fn visit_assignment(&self, ctx: Builder, stmt: &Assignment) -> Result<Builder, VisitError> {}

    fn visit_declaration(&self, ctx: Builder, stmt: &Let) -> Result<Builder, VisitError> {}

    fn visit_expression_statement(
        &self,
        ctx: Builder,
        stmt: &Expression,
    ) -> Result<Builder, VisitError> {
    }

    fn visit_lambda(&self, ctx: Builder, expr: &Lambda) -> Result<Builder, VisitError> {}

    fn visit_reference(
        &self,
        ctx: Builder,
        expr: &MemberExpression,
    ) -> Result<Builder, VisitError> {
    }

    fn visit_call(&self, ctx: Builder, expr: &CallExpression) -> Result<Builder, VisitError> {}

    fn visit_tuple(&self, ctx: Builder, expr: &Tuple) -> Result<Builder, VisitError> {}

    fn visit_number(&self, ctx: Builder, expr: &Number) -> Result<Builder, VisitError> {}

    fn visit_string(&self, ctx: Builder, expr: &StringLiteral) -> Result<Builder, VisitError> {}

    fn visit_unit(&self, ctx: Builder) -> Result<Builder, VisitError> {}

    fn visit_binary(&self, ctx: Builder, expr: &BinaryExpression) -> Result<Builder, VisitError> {}

    fn visit_unary(&self, ctx: Builder, expr: &UnaryExpression) -> Result<Builder, VisitError> {}

    fn visit_if(&self, ctx: Builder, expr: &If) -> Result<Builder, VisitError> {}

    fn visit_table(&self, ctx: Builder, expr: &Table) -> Result<Builder, VisitError> {}

    fn visit_vector(&self, ctx: Builder, expr: &Vector) -> Result<Builder, VisitError> {}

    fn visit_for(&self, ctx: Builder, expr: &For) -> Result<Builder, VisitError> {}

    fn visit_while(&self, ctx: Builder, expr: &While) -> Result<Builder, VisitError> {}

    fn visit_loop(&self, ctx: Builder, expr: &Loop) -> Result<Builder, VisitError> {}

    fn visit_match(&self, ctx: Builder, expr: &Match) -> Result<Builder, VisitError> {}

    fn visit_1tuple(&self, ctx: Builder, expr: &Expression) -> Result<Builder, VisitError> {}

    fn visit_identifier(&self, ctx: Builder, expr: &Identifier) -> Result<Builder, VisitError> {}

    fn visit_do(&self, ctx: Builder, expr: &Do) -> Result<Builder, VisitError> {}
}
