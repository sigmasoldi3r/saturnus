use crate::parser::{Assignment, Class, Declaration, Expression, Function, Return, Script};

#[derive(Debug)]
pub struct VisitorError;

pub trait Visitor<T> {
    fn visit_return(&self, stmt: &Return) -> Result<T, VisitorError>;
    fn visit_class(&self, stmt: &Class) -> Result<T, VisitorError>;
    fn visit_fn(&self, stmt: &Function) -> Result<T, VisitorError>;
    fn visit_assignment(&self, stmt: &Assignment) -> Result<T, VisitorError>;
    fn visit_declaration(&self, stmt: &Declaration) -> Result<T, VisitorError>;
    fn visit_expression_statement(&self, stmt: &Expression) -> Result<T, VisitorError>;
}

pub struct Walker<T>(pub Box<dyn Visitor<T>>);
impl<T> Walker<T> {
    pub fn walk_script(&self, script: Script) -> Result<Vec<T>, VisitorError> {
        script
            .statements
            .iter()
            .map(|stmt| match stmt {
                crate::parser::Statement::If => todo!(),
                crate::parser::Statement::For => todo!(),
                crate::parser::Statement::Loop => todo!(),
                crate::parser::Statement::While => todo!(),
                crate::parser::Statement::Return(e) => self.0.visit_return(e),
                crate::parser::Statement::Class(e) => self.0.visit_class(e),
                crate::parser::Statement::Function(e) => self.0.visit_fn(e),
                crate::parser::Statement::Assignment(e) => self.0.visit_assignment(e),
                crate::parser::Statement::Declaration(e) => self.0.visit_declaration(e),
                crate::parser::Statement::Match => todo!(),
                crate::parser::Statement::Expression(e) => self.0.visit_expression_statement(e),
            })
            .collect()
    }
}
