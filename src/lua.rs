use crate::ast_visitor::Visitor;

pub struct LuaEmitter;

impl Visitor<String> for LuaEmitter {
    fn visit_return(
        &self,
        stmt: &crate::parser::Return,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        todo!()
    }

    fn visit_class(
        &self,
        stmt: &crate::parser::Class,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        todo!()
    }

    fn visit_fn(
        &self,
        stmt: &crate::parser::Function,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        Ok(format!("function {}() end", stmt.name.0))
    }

    fn visit_assignment(
        &self,
        stmt: &crate::parser::Assignment,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        todo!()
    }

    fn visit_declaration(
        &self,
        stmt: &crate::parser::Declaration,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        todo!()
    }

    fn visit_expression_statement(
        &self,
        stmt: &crate::parser::Expression,
    ) -> Result<String, crate::ast_visitor::VisitorError> {
        todo!()
    }
}
