use crate::{
    code::{self, VisitError},
    parser::ast::{self},
};

fn translate_operator(ctx: code::Builder, op: String) -> code::Builder {
    let ctx = ctx.put("__");
    let name = op
        .into_bytes()
        .iter()
        .map(|ch| {
            match ch {
                b'+' => "plus",
                b'-' => "minus",
                b'*' => "times",
                b'/' => "slash",
                b'.' => "dot",
                b'|' => "pipe",
                b'>' => "greater",
                b'<' => "less",
                b'=' => "equals",
                b'?' => "interrogation",
                b'!' => "exclamation",
                b'~' => "tilde",
                b'%' => "percent",
                b'&' => "ampersand",
                b'#' => "bang",
                b'$' => "dollar",
                b'^' => "power",
                _ => panic!(
                    "Error! Unexpected operator {} to be translated as a function!",
                    ch
                ),
            }
            .to_owned()
        })
        .collect::<Vec<String>>()
        .join("_");
    ctx.put(name)
}

pub struct LuaEmitter;

impl LuaEmitter {
    pub fn escape_reference(
        &self,
        ctx: code::Builder,
        ident: &ast::Identifier,
    ) -> Result<code::Builder, VisitError> {
        let ctx = match &ident.0 {
            a if a == "then" => ctx.put("['then']"),
            ident => ctx.put(".").put(ident.clone()),
        };
        Ok(ctx)
    }
    pub fn generate_member_segment<S>(
        &self,
        s: &S,
        ctx: code::Builder,
        elem: &ast::MemberSegment,
    ) -> Result<code::Builder, VisitError>
    where
        S: code::Visitor<code::Builder>,
    {
        match elem {
            ast::MemberSegment::Computed(c) => {
                let ctx = ctx.put("[");
                let ctx = s.visit_expression(ctx, &c)?;
                Ok(ctx.put("]"))
            }
            ast::MemberSegment::IdentifierDynamic(i) => self.escape_reference(ctx, i),
            ast::MemberSegment::IdentifierStatic(_) => Err(VisitError),
        }
    }
    pub fn generate_destructured_assignment(
        &self,
        ctx: code::Builder,
        e: &ast::Destructuring,
    ) -> Result<code::Builder, VisitError> {
        let mut i = 0;
        match e.1 {
            ast::DestructureOrigin::Tuple => e.0.iter().fold(Ok(ctx), |ctx, elem| {
                let ctx = ctx?
                    .line()
                    .put(elem.0.clone())
                    .put(" = __destructure__")
                    .put(format!("._{}", i))
                    .put(";");
                i += 1;
                Ok(ctx)
            }),
            ast::DestructureOrigin::Array => e.0.iter().fold(Ok(ctx), |ctx, elem| {
                let ctx = ctx?
                    .line()
                    .put(elem.0.clone())
                    .put(" = __destructure__")
                    .put(format!("[{}]", i))
                    .put(";");
                i += 1;
                Ok(ctx)
            }),
            ast::DestructureOrigin::Table => e.0.iter().fold(Ok(ctx), |ctx, elem| {
                let ctx = ctx?
                    .line()
                    .put(elem.0.clone())
                    .put(" = __destructure__.")
                    .put(elem.0.clone())
                    .put(";");
                Ok(ctx)
            }),
        }
    }
}

impl code::Visitor<code::Builder> for LuaEmitter {
    fn visit_return(
        &self,
        ctx: code::Builder,
        stmt: &ast::Return,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("return ");
        let ctx = self.visit_expression(ctx, &stmt.value)?;
        Ok(ctx.put(";"))
    }

    fn visit_1tuple(
        &self,
        ctx: code::Builder,
        expr: &ast::Expression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("(");
        let ctx = self.visit_expression(ctx, expr)?;
        Ok(ctx.put(")"))
    }

    fn visit_identifier(
        &self,
        ctx: code::Builder,
        expr: &ast::Identifier,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put(expr.0.clone()))
    }

    fn visit_class(
        &self,
        ctx: code::Builder,
        stmt: &ast::Class,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put(format!("local {} = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!("{}.__meta__ = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!(
                "{}.__meta__.__call = function(self, struct)",
                stmt.name.0.clone()
            ))
            .push()
            .line()
            .put("return setmetatable(struct, self.prototype.__meta__);")
            .pop()
            .unwrap()
            .line()
            .put("end;")
            .line()
            .put(format!("{}.prototype = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!(
                "{}.prototype.__meta__ = {{}};",
                stmt.name.0.clone()
            ))
            .line()
            .put(format!(
                "{}.prototype.__meta__.__index = {}.prototype;",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ))
            .line()
            .put(format!(
                "setmetatable({}, {}.__meta__);",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ));
        let ctx = stmt.fields.iter().fold(Ok(ctx), |ctx, field| {
            let ctx = ctx?.line();
            let ctx = match field {
                ast::ClassField::Method(f) => {
                    let level = if let Some(first) = f.arguments.first() {
                        if first.name.0 == "self" {
                            ".prototype."
                        } else {
                            "."
                        }
                    } else {
                        "."
                    }
                    .to_string();
                    let ctx = ctx
                        .put(stmt.name.0.clone())
                        .put(level)
                        .put(f.name.0.clone())
                        .put(" = ");
                    let ctx = self.visit_lambda(
                        ctx,
                        &ast::Lambda {
                            arguments: f.arguments.clone(),
                            body: ast::ScriptOrExpression::Script(f.body.clone()),
                        },
                    )?;
                    ctx.put(";")
                }
                ast::ClassField::Let(f) => {
                    let ctx = match &f.target {
                        ast::AssignmentTarget::Destructuring(_) => {
                            panic!("Can't destructure that!")
                        }
                        ast::AssignmentTarget::Identifier(e) => ctx.put(format!(
                            "{}.prototype.{} = ",
                            stmt.name.0.clone(),
                            e.0.clone()
                        )),
                    };
                    let ctx = if let Some(value) = f.value.as_ref() {
                        self.visit_expression(ctx, value)?
                    } else {
                        ctx.put("nil")
                    };
                    ctx.put(";")
                }
                ast::ClassField::Operator(f) => {
                    let target = match f.operator.0.as_str() {
                        "+" => "__add",
                        "-" => "__sub",
                        "*" => "__mul",
                        "/" => "__div",
                        "%" => "__mod",
                        "**" => "__pow",
                        "==" => "__eq",
                        "<" => "__lt",
                        "<=" => "__le",
                        "++" => "__concat",
                        "#?" => "__len",
                        _ => todo!(
                            "Operator overload for {:?} operator not supported",
                            f.operator.clone()
                        ),
                    };
                    let ctx = ctx.put(format!(
                        "{}.prototype.__meta__.{} = ",
                        stmt.name.0.clone(),
                        target
                    ));
                    let ctx = self.visit_lambda(
                        ctx,
                        &ast::Lambda {
                            arguments: f.arguments.clone(),
                            body: ast::ScriptOrExpression::Script(f.body.clone()),
                        },
                    )?;
                    ctx.put(";")
                }
            };
            let ctx = stmt.decorators.iter().fold(Ok(ctx), |ctx, dec| {
                let ctx = ctx?.line();
                let ctx = self.visit_call(ctx, &dec.target)?;
                let ctx = ctx.put(format!(
                    "({}, \"{}\");",
                    stmt.name.0.clone(),
                    stmt.name.0.clone()
                ));
                Ok(ctx)
            })?;
            Ok(ctx)
        })?;
        Ok(ctx)
    }

    fn visit_fn(
        &self,
        ctx: code::Builder,
        stmt: &ast::Function,
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
        let ctx = if stmt.arguments.len() > 0 {
            ctx.put(", ...")
        } else {
            ctx.put("...")
        };
        let ctx = ctx.put(")").push();
        let ctx = self.visit_script(ctx, &stmt.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        let ctx = stmt.decorators.iter().fold(Ok(ctx), |ctx, dec| {
            let ctx = ctx?.line();
            let ctx = self.visit_call(ctx, &dec.target)?;
            let ctx = ctx.put(format!(
                "({}, \"{}\");",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ));
            Ok(ctx)
        })?;
        Ok(ctx)
    }

    fn visit_assignment(
        &self,
        ctx: code::Builder,
        stmt: &ast::Assignment,
    ) -> Result<code::Builder, code::VisitError> {
        let segment = self
            .visit_reference(ctx.clone_like(), &stmt.target)?
            .collect();
        let ctx = ctx.line().put(segment).put(" = ");
        let ctx = if let Some(extra) = stmt.extra.as_ref() {
            let ast::Assignment { target, value, .. } = stmt.clone();
            self.visit_binary(
                ctx,
                &ast::BinaryExpression {
                    left: ast::Expression::Reference(Box::new(target)),
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
        stmt: &ast::Let,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match &stmt.target {
            ast::AssignmentTarget::Destructuring(e) => {
                let ctx = ctx.line().put("local ");
                let ctx = ctx.put(e.0.get(0).unwrap().0.clone());
                let ctx =
                    e.0.iter()
                        .skip(1)
                        .fold(ctx, |ctx, elem| ctx.put(", ").put(elem.0.clone()));
                let ctx = ctx
                    .put(";")
                    .line()
                    .put("do")
                    .push()
                    .line()
                    .put("local __destructure__ = ");
                let ctx = self
                    .visit_expression(ctx, stmt.value.as_ref().unwrap())?
                    .put(";");
                self.generate_destructured_assignment(ctx, &e)?
                    .pop()
                    .unwrap()
                    .line()
                    .put("end")
            }
            ast::AssignmentTarget::Identifier(e) => {
                let ctx = ctx.line().put("local ").put(e.0.clone()).put(" = ");
                let ctx = self.visit_expression(ctx, stmt.value.as_ref().unwrap())?;
                ctx.put(";")
            }
        };
        Ok(ctx)
    }

    fn visit_expression_statement(
        &self,
        ctx: code::Builder,
        stmt: &ast::Expression,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(self.visit_expression(ctx.line(), stmt)?.put(";"))
    }

    fn visit_lambda(
        &self,
        ctx: code::Builder,
        expr: &ast::Lambda,
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
        let ctx = if expr.arguments.len() > 0 {
            ctx.put(", ...")
        } else {
            ctx.put("...")
        };
        let ctx = ctx.put(")").push();
        let ctx = match &expr.body {
            ast::ScriptOrExpression::Script(e) => self.visit_script(ctx, e)?,
            ast::ScriptOrExpression::Expression(e) => self
                .visit_expression(ctx.line().put("return "), e)
                .map(|b| b.put(";"))?,
        };
        Ok(ctx.pop().unwrap().line().put("end"))
    }

    fn visit_reference(
        &self,
        ctx: code::Builder,
        expr: &ast::MemberExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = self.visit_expression(ctx, &expr.head)?;
        let ctx = expr.tail.iter().fold(Ok(ctx), |ctx, elem| {
            self.generate_member_segment(self, ctx?, elem)
        })?;
        Ok(ctx)
    }

    fn enter_script(
        &self,
        ctx: code::Builder,
        _script: &crate::parser::Script,
    ) -> Result<code::Builder, VisitError> {
        Ok(ctx.line().put("local argv = {...};"))
    }

    fn visit_call(
        &self,
        ctx: code::Builder,
        expr: &ast::CallExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = if let Some(callee) = expr.head.callee.clone() {
            let ctx = self.visit_expression(ctx, &callee.head)?;
            let ctx = callee
                .tail
                .iter()
                .rev()
                .skip(1)
                .rev()
                .fold(Ok(ctx), |ctx, elem| {
                    let ctx = ctx?;
                    let ctx = match elem {
                        ast::MemberSegment::Computed(c) => {
                            let ctx = ctx.put("[");
                            let ctx = self.visit_expression(ctx, &c)?;
                            ctx.put("]")
                        }
                        ast::MemberSegment::IdentifierDynamic(c) => ctx.put(".").put(c.0.clone()),
                        ast::MemberSegment::IdentifierStatic(_) => Err(VisitError)?,
                    };
                    Ok(ctx)
                })?;
            let ctx = if let Some(last) = callee.tail.last() {
                match last {
                    ast::MemberSegment::Computed(c) => {
                        let ctx = ctx.put("[");
                        let ctx = self.visit_expression(ctx, &c)?;
                        ctx.put("]")
                    }
                    ast::MemberSegment::IdentifierDynamic(c) => ctx.put(":").put(c.0.clone()),
                    ast::MemberSegment::IdentifierStatic(c) => ctx.put(".").put(c.0.clone()),
                }
            } else {
                ctx
            };
            ctx
        } else {
            ctx
        };
        let ctx = ctx.put("(");
        let ctx = if let Some(first) = expr.head.arguments.first() {
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr
            .head
            .arguments
            .iter()
            .skip(1)
            .fold(Ok(ctx), |ctx, elem| {
                let ctx = ctx?.put(", ");
                self.visit_expression(ctx, elem)
            })?;
        let ctx = ctx.put(")");
        let ctx = expr.tail.iter().fold(Ok(ctx), |ctx, elem| match elem {
            ast::CallExpressionVariant::Call(c) => self.visit_call(
                ctx?,
                &ast::CallExpression {
                    head: c.clone(),
                    tail: vec![],
                },
            ),
            ast::CallExpressionVariant::Member(m) => match m {
                ast::MemberSegment::Computed(c) => {
                    let ctx = ctx?.put("[");
                    let ctx = self.visit_expression(ctx, &c)?;
                    Ok(ctx.put("]"))
                }
                ast::MemberSegment::IdentifierStatic(i) => Ok(ctx?.put(".").put(i.0.clone())),
                ast::MemberSegment::IdentifierDynamic(i) => Ok(ctx?.put(":").put(i.0.clone())),
            },
        })?;
        Ok(ctx)
    }

    fn visit_tuple(
        &self,
        ctx: code::Builder,
        expr: &ast::Tuple,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some(first) = expr.0.first().as_ref() {
            let ctx = ctx.put(format!("_0 = "));
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr
            .0
            .iter()
            .skip(1)
            .fold(Ok((ctx, 1_u16)), |ctx, value| {
                let (ctx, i) = ctx?;
                let ctx = ctx.put(format!(", _{} = ", i));
                let ctx = self.visit_expression(ctx, value)?;
                Ok((ctx, i + 1))
            })?
            .0;
        let ctx = ctx.put("}");
        Ok(ctx)
    }

    fn visit_number(
        &self,
        ctx: code::Builder,
        expr: &ast::Number,
    ) -> Result<code::Builder, code::VisitError> {
        let repr = match expr {
            ast::Number::Float(e) => e.to_string(),
            ast::Number::Integer(e) => e.to_string(),
        };
        Ok(ctx.put(repr))
    }

    fn visit_string(
        &self,
        ctx: code::Builder,
        expr: &ast::StringLiteral,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match expr {
            ast::StringLiteral::Double(s) => ctx.put("\"").put(s.clone()).put("\""),
            ast::StringLiteral::Single(s) => ctx.put("'").put(s.clone()).put("'"),
            ast::StringLiteral::Special(_) => todo!(),
        };
        Ok(ctx)
    }

    fn visit_unit(&self, ctx: code::Builder) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put("nil"))
    }

    fn visit_binary(
        &self,
        ctx: code::Builder,
        expr: &ast::BinaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let op = expr.operator.0.as_str();
        // Extra logic (Indirect operation expansion)
        if op == "??" {
            let ctx = self.visit_expression(ctx, &expr.left)?.put(" == nil and ");
            let ctx = self.visit_expression(ctx, &expr.right)?.put(" or ");
            return self.visit_expression(ctx, &expr.left);
        } else if op == "?:" {
            let ctx = self.visit_expression(ctx, &expr.left)?.put(" or ");
            return self.visit_expression(ctx, &expr.right);
        }
        // Translate native operators.
        match op {
            // Basic math
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            // Comparison
            | ">"
            | ">="
            | "<"
            | "<="
            | "=="
            | "~="
            // Logic
            | "not"
            | "and"
            | "or"
            // Logic
            | "&"
            | "|"
            | "<<"
            | "<<<"
            | ">>"
            | ">>>"
                => {
                let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
                let ctx = ctx.put(op.to_owned());
                self.visit_expression(ctx.put(" "), &expr.right)
            }
            "++"=> {
                // Native-to-native operator translation
                let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
                let ctx = ctx.put("..".to_owned());
                self.visit_expression(ctx.put(" "), &expr.right)
            }
            _=> {
                // Direct function translation
                let ctx = translate_operator(ctx, op.to_owned()).put("(");
                let ctx = self.visit_expression(ctx, &expr.left)?.put(", ");
                let ctx = self.visit_expression(ctx, &expr.right)?.put(")");
                Ok(ctx)
            }
        }
    }

    fn visit_unary(
        &self,
        ctx: code::Builder,
        expr: &ast::UnaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match expr.operator.clone().0.as_str() {
            "-" => ctx.put("-"),
            "not" => ctx.put("not "),
            "#?" => ctx.put("#"),
            op => todo!("Unary operator {:?} not supported!", op),
        };
        let ctx = self.visit_expression(ctx, &expr.expression)?;
        Ok(ctx)
    }

    fn visit_if(
        &self,
        ctx: code::Builder,
        expr: &ast::If,
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
        expr: &ast::Table,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some((k, v)) = expr.key_values.first() {
            match k {
                ast::TableKeyExpression::Identifier(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ");
                    self.visit_expression(ctx, &v.clone().unwrap())
                }
                ast::TableKeyExpression::Expression(k) => {
                    let ctx = self.visit_expression(ctx, &k)?.put(" = ");
                    self.visit_expression(ctx, &v.clone().unwrap())
                }
                ast::TableKeyExpression::Implicit(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ").put(k.0.clone());
                    Ok(ctx)
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
                    ast::TableKeyExpression::Identifier(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ");
                        self.visit_expression(ctx, &v.clone().unwrap())
                    }
                    ast::TableKeyExpression::Expression(k) => {
                        let ctx = ctx.put("[");
                        let ctx = self.visit_expression(ctx, &k)?.put("] = ");
                        self.visit_expression(ctx, &v.clone().unwrap())
                    }
                    ast::TableKeyExpression::Implicit(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ").put(k.0.clone());
                        Ok(ctx)
                    }
                }
            })?;
        Ok(ctx.put("}"))
    }

    fn visit_vector(
        &self,
        ctx: code::Builder,
        expr: &ast::Vector,
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

    fn visit_for(
        &self,
        ctx: code::Builder,
        expr: &ast::For,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match &expr.handler {
            ast::AssignmentTarget::Destructuring(e) => {
                let ctx = ctx.line().put("for __destructuring__ in ");
                let ctx = self.visit_expression(ctx, &expr.target)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.generate_destructured_assignment(ctx, &e)?;
                let ctx = self.visit_script(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
            ast::AssignmentTarget::Identifier(e) => {
                let ctx = ctx.line().put(format!("for {} in ", e.0.clone()));
                let ctx = self.visit_expression(ctx, &expr.target)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.visit_script(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
        };
        Ok(ctx)
    }

    fn visit_while(
        &self,
        ctx: code::Builder,
        expr: &ast::While,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match &expr.condition {
            ast::ExpressionOrLet::Expression(e) => {
                let ctx = ctx.line().put("while ");
                let ctx = self.visit_expression(ctx, e)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.visit_script(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
            ast::ExpressionOrLet::Let(e) => {
                if let ast::AssignmentTarget::Identifier(id) = &e.target {
                    let ctx = ctx.line().put("do").push();
                    let ctx = self.visit_declaration(ctx, e)?;
                    let ctx = ctx.line().put(format!("while {} do", id.0.clone())).push();
                    let ctx = self.visit_script(ctx, &expr.body)?;
                    let ctx = self.visit_assignment(
                        ctx,
                        &ast::Assignment {
                            target: ast::MemberExpression {
                                head: ast::Expression::Identifier(id.clone()),
                                tail: vec![],
                            },
                            value: e.value.clone().unwrap(),
                            extra: None,
                        },
                    )?;
                    ctx.pop()
                        .unwrap()
                        .line()
                        .put("end")
                        .pop()
                        .unwrap()
                        .line()
                        .put("end")
                } else {
                    panic!("Destructured while-let not supported");
                }
            }
        };
        Ok(ctx)
    }

    fn visit_loop(
        &self,
        ctx: code::Builder,
        expr: &ast::Loop,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("while true do").push();
        let ctx = self.visit_script(ctx, &expr.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_match(
        &self,
        _ctx: code::Builder,
        _expr: &ast::Match,
    ) -> Result<code::Builder, code::VisitError> {
        todo!("Match code generation not implemented yet.")
    }
}
