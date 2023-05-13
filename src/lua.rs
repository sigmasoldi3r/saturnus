use crate::code::{self};

pub struct LuaEmitter;

impl code::Visitor<code::Builder> for LuaEmitter {
    fn visit_return(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Return,
    ) -> Result<code::Builder, code::VisitError> {
        let context = context.line().put("return ");
        let context = self.visit_expression(context, &stmt.value)?;
        Ok(context.put(";"))
    }

    fn visit_class(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Class,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(context
            .line()
            .put("local ")
            .put(stmt.name.0.clone())
            .put(" = {};"))
    }

    fn visit_fn(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Function,
    ) -> Result<code::Builder, code::VisitError> {
        let context = context
            .line()
            .put("local function ")
            .put(stmt.name.0.clone())
            .put("()")
            .push();
        let context = self.visit_script(context, &stmt.body)?;
        let context = context.pop().unwrap().line().put("end");
        Ok(context)
    }

    fn visit_assignment(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Assignment,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_declaration(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Declaration,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_expression_statement(
        &self,
        context: code::Builder,
        stmt: &crate::parser::Expression,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(self.visit_expression(context.line(), stmt)?.put(";"))
    }

    fn visit_lambda(
        &self,
        context: code::Builder,
        expr: &crate::parser::Lambda,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_reference(
        &self,
        context: code::Builder,
        expr: &crate::parser::DotExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let context = context.put(expr.0.first().unwrap().0.clone());
        let context = expr.0.iter().skip(1).fold(context, |context, ident| {
            context.put(".").put(ident.0.clone())
        });
        Ok(context)
    }

    fn visit_call(
        &self,
        context: code::Builder,
        expr: &crate::parser::CallExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let context = self.visit_reference(context, &expr.target)?.put("(");
        let context = if let Some(first) = expr.arguments.0.first() {
            self.visit_expression(context, first)?
        } else {
            context
        };
        let context = expr
            .arguments
            .0
            .iter()
            .skip(1)
            .fold(Ok(context), |context, expr| {
                self.visit_expression(context.map(|b| b.put(", "))?, expr)
            })?;
        Ok(context.put(")"))
    }

    fn visit_tuple(
        &self,
        context: code::Builder,
        expr: &crate::parser::Tuple,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_number(
        &self,
        context: code::Builder,
        expr: &crate::parser::Number,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_string(
        &self,
        context: code::Builder,
        expr: &String,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(context.put("\"").put(expr.clone()).put("\""))
    }

    fn visit_unit(&self, context: code::Builder) -> Result<code::Builder, code::VisitError> {
        Ok(context.put("nil"))
    }
}
