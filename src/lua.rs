use crate::{
    code::{self},
    parser::{
        Assignment, BinaryExpression, DotExpression, Identifier, Lambda, LambdaBody, Operator,
    },
};

pub struct LuaEmitter;

impl code::Visitor<code::Builder> for LuaEmitter {
    fn visit_return(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Return,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("return ");
        let ctx = self.visit_expression(ctx, &stmt.value)?;
        Ok(ctx.put(";"))
    }

    fn visit_class(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Class,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put("local ")
            .put(stmt.name.0.clone())
            .put(" = {};")
            .line()
            .put(stmt.name.0.clone())
            .put(".__meta__ = {};")
            .line()
            .put(stmt.name.0.clone())
            .put(".__index = ")
            .put(stmt.name.0.clone())
            .put(";")
            .line()
            .put(stmt.name.0.clone())
            .put(".__meta__.__call = function(self, struct)")
            .push()
            .line()
            .put("return setmetatable(struct, ")
            .put(stmt.name.0.clone())
            .put(");")
            .pop()
            .unwrap()
            .line()
            .put("end;")
            .line()
            .put("setmetatable(")
            .put(stmt.name.0.clone())
            .put(", ")
            .put(stmt.name.0.clone())
            .put(".__meta__);");
        let ctx = stmt.fields.iter().fold(Ok(ctx), |ctx, field| {
            let ctx = ctx?.line();
            let ctx = match field {
                crate::parser::ClassField::Method(f) => {
                    let ctx = ctx
                        .put(stmt.name.0.clone())
                        .put(".")
                        .put(f.name.0.clone())
                        .put(" = ");
                    let ctx = self.visit_lambda(
                        ctx,
                        &Lambda {
                            arguments: f.arguments.clone(),
                            body: LambdaBody::Complex(f.body.clone()),
                        },
                    )?;
                    ctx.put(";")
                }
                crate::parser::ClassField::Let(f) => {
                    let ctx = ctx
                        .put(stmt.name.0.clone())
                        .put(".")
                        .put(f.target.0.clone())
                        .put(" = ");
                    let ctx = if let Some(value) = f.value.as_ref() {
                        self.visit_expression(ctx, value)?
                    } else {
                        ctx.put("nil")
                    };
                    ctx.put(";")
                }
                crate::parser::ClassField::Operator(f) => {
                    todo!("Operator overload not implemented yet")
                }
            };
            Ok(ctx)
        })?;
        Ok(ctx)
    }

    fn visit_fn(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Function,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put("local function ")
            .put(stmt.name.0.clone())
            .put("(");
        let ctx = if let Some(first) = stmt.arguments.first() {
            ctx.put(first.name.0.clone())
        } else {
            ctx
        };
        let ctx = stmt
            .arguments
            .iter()
            .skip(1)
            .fold(ctx, |ctx, ident| ctx.put(", ").put(ident.name.0.clone()));
        let ctx = ctx.put(")").push();
        let ctx = self.visit_script(ctx, &stmt.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_assignment(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Assignment,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put(stmt.target.0.clone());
        let ctx = ctx.put(" = ");
        let ctx = if let Some(extra) = stmt.extra.as_ref() {
            let Assignment { target, value, .. } = stmt.clone();
            self.visit_binary(
                ctx,
                &BinaryExpression {
                    left: crate::parser::Expression::Reference(DotExpression(vec![target])),
                    operator: extra.clone(),
                    right: value,
                },
            )
        } else {
            self.visit_expression(ctx, &stmt.value)
        }?;
        let ctx = ctx.put(";");
        Ok(ctx)
    }

    fn visit_declaration(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Let,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put("local ")
            .put(stmt.target.0.clone())
            .put(" = ");
        let ctx = self.visit_expression(ctx, stmt.value.as_ref().unwrap())?;
        let ctx = ctx.put(";");
        Ok(ctx)
    }

    fn visit_expression_statement(
        &self,
        ctx: code::Builder,
        stmt: &crate::parser::Expression,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(self.visit_expression(ctx.line(), stmt)?.put(";"))
    }

    fn visit_lambda(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::Lambda,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("function(");
        let ctx = if let Some(first) = expr.arguments.first() {
            ctx.put(first.name.0.clone())
        } else {
            ctx
        };
        let ctx = expr
            .arguments
            .iter()
            .skip(1)
            .fold(ctx, |ctx, ident| ctx.put(", ").put(ident.name.0.clone()));
        let ctx = ctx.put(")").push();
        let ctx = match &expr.body {
            crate::parser::LambdaBody::Complex(e) => self.visit_script(ctx, e)?,
            crate::parser::LambdaBody::Simple(e) => self
                .visit_expression(ctx.line().put("return "), e)
                .map(|b| b.put(";"))?,
        };
        Ok(ctx.pop().unwrap().line().put("end"))
    }

    fn visit_reference(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::DotExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put(expr.0.first().unwrap().0.clone());
        let ctx = expr
            .0
            .iter()
            .skip(1)
            .fold(ctx, |ctx, ident| ctx.put(".").put(ident.0.clone()));
        Ok(ctx)
    }

    fn visit_call(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::CallExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = self.visit_reference(ctx, &expr.target)?.put("(");
        let ctx = if let Some(first) = expr.arguments.0.first() {
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr.arguments.0.iter().skip(1).fold(Ok(ctx), |ctx, expr| {
            self.visit_expression(ctx.map(|b| b.put(", "))?, expr)
        })?;
        Ok(ctx.put(")"))
    }

    fn visit_tuple(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::Tuple,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }

    fn visit_number(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::Number,
    ) -> Result<code::Builder, code::VisitError> {
        let repr = match expr {
            crate::parser::Number::Float(e) => e.to_string(),
            crate::parser::Number::Integer(e) => e.to_string(),
        };
        Ok(ctx.put(repr))
    }

    fn visit_string(
        &self,
        ctx: code::Builder,
        expr: &String,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put("\"").put(expr.clone()).put("\""))
    }

    fn visit_unit(&self, ctx: code::Builder) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put("nil"))
    }

    fn visit_binary(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::BinaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
        let ctx = match expr.operator {
            // Basic math
            Operator::Plus => ctx.put("+"),
            Operator::Minus => ctx.put("-"),
            Operator::Product => ctx.put("*"),
            Operator::Quotient => ctx.put("/"),
            Operator::Remainder => ctx.put("%"),
            Operator::Power => ctx.put("**"),
            // Comparison
            Operator::Greater => ctx.put(">"),
            Operator::GreaterEqual => ctx.put(">="),
            Operator::Less => ctx.put("<"),
            Operator::LessEqual => ctx.put("<="),
            Operator::Equal => ctx.put("=="),
            Operator::NotEqual => ctx.put("~="),
            _ => todo!("Binary operator not supported!"),
        };
        let ctx = self.visit_expression(ctx.put(" "), &expr.right)?;
        Ok(ctx)
    }

    fn visit_unary(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::UnaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match expr.operator {
            Operator::Minus => ctx.put("-"),
            _ => todo!("Unary operator not supported!"),
        };
        let ctx = self.visit_expression(ctx, &expr.expression)?;
        Ok(ctx)
    }

    fn visit_if(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::If,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("if ");
        let ctx = self.visit_expression(ctx, &expr.condition)?;
        let ctx = ctx.put(" then").push();
        let ctx = self.visit_script(ctx, &expr.body)?;
        let ctx = expr.branches.iter().fold(Ok(ctx), |ctx, (c, s)| {
            let ctx = ctx?.pop().unwrap().line().put("elseif ");
            let ctx = self.visit_expression(ctx, c)?;
            let ctx = ctx.put(" then").push();
            let ctx = self.visit_script(ctx, s)?;
            Ok(ctx)
        })?;
        let ctx = if let Some(eb) = expr.else_branch.as_ref() {
            let ctx = ctx.pop().unwrap().line().put("else").push();
            self.visit_script(ctx, eb)?
        } else {
            ctx
        };
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_table(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::Table,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some((k, v)) = expr.key_values.first() {
            match k {
                crate::parser::TableKeyExpression::Identifier(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ");
                    self.visit_expression(ctx, v)
                }
                crate::parser::TableKeyExpression::Expression(k) => {
                    let ctx = self.visit_expression(ctx, k)?.put(" = ");
                    self.visit_expression(ctx, v)
                }
                crate::parser::TableKeyExpression::Implicit(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ");
                    self.visit_expression(
                        ctx,
                        &crate::parser::Expression::Reference(DotExpression(vec![k.clone()])),
                    )
                }
            }?
        } else {
            ctx
        };
        let ctx = expr
            .key_values
            .iter()
            .skip(1)
            .fold(Ok(ctx), |ctx, (k, v)| {
                let ctx = ctx?.put(", ");
                match k {
                    crate::parser::TableKeyExpression::Identifier(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ");
                        self.visit_expression(ctx, v)
                    }
                    crate::parser::TableKeyExpression::Expression(k) => {
                        let ctx = ctx.put("[");
                        let ctx = self.visit_expression(ctx, k)?.put("] = ");
                        self.visit_expression(ctx, v)
                    }
                    crate::parser::TableKeyExpression::Implicit(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ");
                        self.visit_expression(
                            ctx,
                            &crate::parser::Expression::Reference(DotExpression(vec![k.clone()])),
                        )
                    }
                }
            })?;
        Ok(ctx.put("}"))
    }

    fn visit_vector(
        &self,
        ctx: code::Builder,
        expr: &crate::parser::Vector,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some(first) = expr.expressions.first() {
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr.expressions.iter().skip(1).fold(Ok(ctx), |ctx, v| {
            let ctx = ctx?.put(", ");
            let ctx = self.visit_expression(ctx, v)?;
            Ok(ctx)
        })?;
        Ok(ctx.put("}"))
    }
}
