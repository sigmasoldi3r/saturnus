use std::error::Error;

use crate::parser::ast::*;

use super::{
    builder::Builder,
    walker::{AstWalker, Result},
};

#[derive(Debug)]
struct ConstructionError(Box<dyn Error>);

pub trait CodeBuilder {
    // Classes
    fn build_class(&self, ctx: Builder, target: &Class) -> Result;
    fn build_class_method(&self, ctx: Builder, target: &Function) -> Result;
    fn build_class_value(&self, ctx: Builder, target: &Let) -> Result;
    fn build_class_decorator(&self, ctx: Builder, target: &Decorator) -> Result;
    fn build_class_field_decorator(&self, ctx: Builder, target: &Decorator) -> Result;

    // Functions
    fn build_function(&self, ctx: Builder, target: &Function) -> Result;
    fn build_function_decorator(&self, ctx: Builder, target: &Decorator) -> Result;
    fn build_macro_function(&self, ctx: Builder, target: &Function) -> Result;
    fn build_native_function(&self, ctx: Builder, target: &Function) -> Result;

    // Lambdas and variables
    fn build_lambda(&self, ctx: Builder, target: &Lambda) -> Result;
    fn build_let(&self, ctx: Builder, target: &Let) -> Result;
    fn build_use_statement(&self, ctx: Builder, target: &Let) -> Result;
    fn build_use_expression(&self, ctx: Builder, target: &Let) -> Result;
    fn build_return(&self, ctx: Builder, target: &Return) -> Result;
    fn build_unit(&self, ctx: Builder, target: ()) -> Result;

    // Conditional and looping blocks
    fn build_if(&self, ctx: Builder, target: &If) -> Result;
    fn build_for(&self, ctx: Builder, target: &For) -> Result;
    fn build_while(&self, ctx: Builder, target: &While) -> Result;
    fn build_loop(&self, ctx: Builder, target: &Loop) -> Result;
    fn build_while_let(&self, ctx: Builder, target: &While) -> Result;

    // Script blocks
    fn build_block(&self, ctx: Builder, target: &Class) -> Result;
    fn build_script(&self, ctx: Builder, target: &Class) -> Result;
}

pub struct CodeEmitter {
    pub builder: Box<dyn CodeBuilder>,
}

impl AstWalker for CodeEmitter {
    fn visit_return(&self, ctx: Builder, stmt: &Return) -> Result {
        self.builder.build_return(ctx, stmt)
    }

    fn visit_class(&self, ctx: Builder, stmt: &Class) -> Result {
        let ctx = self.builder.build_class(ctx, stmt)?;
        let ctx = stmt.fields.iter().fold(Ok(ctx), |ctx, field| match field {
            ClassField::Method(field) => self.builder.build_class_method(ctx?, field),
            ClassField::Let(field) => self.builder.build_class_value(ctx?, field),
        })?;
        let ctx = stmt.decorators.iter().fold(Ok(ctx), |ctx, dec| {
            self.builder.build_class_decorator(ctx?, dec)
        })?;
    }

    fn visit_fn(&self, ctx: Builder, stmt: &Function) -> Result {
        todo!()
    }

    fn visit_assignment(&self, ctx: Builder, stmt: &Assignment) -> Result {
        todo!()
    }

    fn visit_declaration(&self, ctx: Builder, stmt: &Let) -> Result {
        todo!()
    }

    fn visit_expression_statement(&self, ctx: Builder, stmt: &Expression) -> Result {
        todo!()
    }

    fn visit_lambda(&self, ctx: Builder, expr: &Lambda) -> Result {
        todo!()
    }

    fn visit_reference(&self, ctx: Builder, expr: &MemberExpression) -> Result {
        todo!()
    }

    fn visit_call(&self, ctx: Builder, expr: &CallExpression) -> Result {
        todo!()
    }

    fn visit_tuple(&self, ctx: Builder, expr: &Tuple) -> Result {
        todo!()
    }

    fn visit_number(&self, ctx: Builder, expr: &Number) -> Result {
        todo!()
    }

    fn visit_string(&self, ctx: Builder, expr: &StringLiteral) -> Result {
        todo!()
    }

    fn visit_unit(&self, ctx: Builder) -> Result {
        todo!()
    }

    fn visit_binary(&self, ctx: Builder, expr: &BinaryExpression) -> Result {
        todo!()
    }

    fn visit_unary(&self, ctx: Builder, expr: &UnaryExpression) -> Result {
        todo!()
    }

    fn visit_if(&self, ctx: Builder, expr: &If) -> Result {
        todo!()
    }

    fn visit_table(&self, ctx: Builder, expr: &Table) -> Result {
        todo!()
    }

    fn visit_vector(&self, ctx: Builder, expr: &Vector) -> Result {
        todo!()
    }

    fn visit_for(&self, ctx: Builder, expr: &For) -> Result {
        todo!()
    }

    fn visit_while(&self, ctx: Builder, expr: &While) -> Result {
        todo!()
    }

    fn visit_loop(&self, ctx: Builder, expr: &Loop) -> Result {
        todo!()
    }

    fn visit_match(&self, ctx: Builder, expr: &Match) -> Result {
        todo!()
    }

    fn visit_1tuple(&self, ctx: Builder, expr: &Expression) -> Result {
        todo!()
    }

    fn visit_identifier(&self, ctx: Builder, expr: &Identifier) -> Result {
        todo!()
    }

    fn visit_do(&self, ctx: Builder, expr: &Do) -> Result {
        todo!()
    }
}
