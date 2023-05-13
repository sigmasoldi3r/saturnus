use crate::parser::{
    Assignment, BinaryExpression, CallExpression, Class, Declaration, DotExpression, Expression,
    Function, If, Lambda, Number, Return, Script, Tuple, UnaryExpression,
};

#[derive(Debug)]
pub struct VisitError;

pub trait Visitor<T> {
    // Those need to be implemented explicitly by the user:
    fn visit_return(&self, context: T, stmt: &Return) -> Result<T, VisitError>;
    fn visit_class(&self, context: T, stmt: &Class) -> Result<T, VisitError>;
    fn visit_fn(&self, context: T, stmt: &Function) -> Result<T, VisitError>;
    fn visit_assignment(&self, context: T, stmt: &Assignment) -> Result<T, VisitError>;
    fn visit_declaration(&self, context: T, stmt: &Declaration) -> Result<T, VisitError>;
    fn visit_expression_statement(&self, context: T, stmt: &Expression) -> Result<T, VisitError>;
    fn visit_lambda(&self, context: T, expr: &Lambda) -> Result<T, VisitError>;
    fn visit_reference(&self, context: T, expr: &DotExpression) -> Result<T, VisitError>;
    fn visit_call(&self, context: T, expr: &CallExpression) -> Result<T, VisitError>;
    fn visit_tuple(&self, context: T, expr: &Tuple) -> Result<T, VisitError>;
    fn visit_number(&self, context: T, expr: &Number) -> Result<T, VisitError>;
    fn visit_string(&self, context: T, expr: &String) -> Result<T, VisitError>;
    fn visit_unit(&self, context: T) -> Result<T, VisitError>;
    fn visit_binary(&self, context: T, expr: &BinaryExpression) -> Result<T, VisitError>;
    fn visit_unary(&self, context: T, expr: &UnaryExpression) -> Result<T, VisitError>;
    fn visit_if(&self, context: T, expr: &If) -> Result<T, VisitError>;

    // Generically implementable matching patterns:
    fn visit_expression(&self, context: T, expression: &Expression) -> Result<T, VisitError> {
        match expression {
            Expression::Lambda(e) => self.visit_lambda(context, e),
            Expression::Reference(e) => self.visit_reference(context, e),
            Expression::Call(e) => self.visit_call(context, e),
            Expression::Tuple(e) => self.visit_tuple(context, e),
            Expression::Number(e) => self.visit_number(context, e),
            Expression::String(e) => self.visit_string(context, e),
            Expression::Unit => self.visit_unit(context),
            Expression::Binary(e) => self.visit_binary(context, e),
            Expression::Unary(e) => self.visit_unary(context, e),
        }
    }
    fn visit_script(&self, context: T, script: &Script) -> Result<T, VisitError> {
        script
            .statements
            .iter()
            .fold(Ok(context), |context, stmt| match stmt {
                crate::parser::Statement::If(e) => self.visit_if(context?, e),
                crate::parser::Statement::For => todo!(),
                crate::parser::Statement::Loop => todo!(),
                crate::parser::Statement::While => todo!(),
                crate::parser::Statement::Return(e) => self.visit_return(context?, e),
                crate::parser::Statement::Class(e) => self.visit_class(context?, e),
                crate::parser::Statement::Function(e) => self.visit_fn(context?, e),
                crate::parser::Statement::Assignment(e) => self.visit_assignment(context?, e),
                crate::parser::Statement::Declaration(e) => self.visit_declaration(context?, e),
                crate::parser::Statement::Match => todo!(),
                crate::parser::Statement::Expression(e) => {
                    self.visit_expression_statement(context?, e)
                }
            })
    }
}

pub struct UnevenIndentationError;
impl std::fmt::Debug for UnevenIndentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Uneven Indentation: Attempting to pop furhter than 0!")
            .finish()
    }
}

/// # String code builder
///
/// This simple immutable builder accounts for raw strings (So it is agnostic
/// from the targetted output), but retains indentation aspects.
///
/// This means that you have an indentation stack, with it's state retained
/// between calls, without having to store it in your code emitter.
///
/// Each call, consumes the builder and returns an extended version of it.
/// If you want to preserve the state, clone the structure by calling `.clone()`
/// explicitly.
///
/// Example:
/// ```rs
/// let out = Builder::new("  ")
///     .put("hello")
///     .push().line()
///     .put("my")
///     .pop().unwrap().line()
///     .put("world!")
///     .collect()
/// ```
/// Yields:
/// ```
/// hello
///   my
/// world
/// ```
#[derive(Clone)]
pub struct Builder {
    level: u16,
    indent: String,
    buffer: String,
}
impl Builder {
    pub fn new<T>(indent: T) -> Self
    where
        T: Into<String>,
    {
        Builder {
            level: 0,
            indent: indent.into(),
            buffer: Default::default(),
        }
    }
    pub fn collect(self) -> String {
        self.buffer
    }
    pub fn push(self) -> Self {
        Builder {
            level: self.level + 1,
            ..self
        }
    }
    pub fn pop(self) -> Result<Self, UnevenIndentationError> {
        if self.level == 0 {
            Err(UnevenIndentationError)
        } else {
            Ok(Builder {
                level: self.level - 1,
                ..self
            })
        }
    }
    pub fn put<T>(self, fragment: T) -> Self
    where
        T: Into<String>,
    {
        Builder {
            buffer: format!("{}{}", self.buffer, fragment.into()),
            ..self
        }
    }
    pub fn line(self) -> Self {
        Builder {
            buffer: format!("{}\n{}", self.buffer, self.indent.repeat(self.level.into())),
            ..self
        }
    }
}
