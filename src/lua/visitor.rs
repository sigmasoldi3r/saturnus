use crate::{
    code::{
        ast_visitor::{Result, Visitor},
        builder::Builder,
    },
    parser::{
        ast::{self, CallExpression, Identifier, MemberExpression},
        helpers::generate_operator_function_name,
    },
};

#[derive(Debug)]
struct BadCode;
impl std::fmt::Display for BadCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Bad code")
    }
}
impl std::error::Error for BadCode {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

fn escape_string(str: String) -> String {
    return str.replace("\n", "\\n");
}

pub struct LuaEmitter {
    // pub module_mapping: HashMap<String, PathBuf>,
}

impl LuaEmitter {
    pub fn new() -> Self {
        Self {
            // module_mapping: HashMap::new(),
        }
    }
    // pub fn map_module_path(&self, segments: &Vec<String>) -> String {
    //     let path = segments.join(".");
    //     let re = regex::Regex::new(r"\.saturn$").unwrap();
    //     if let Some(found) = self.module_mapping.get(&path) {
    //         let found = found
    //             .iter()
    //             .map(|p| p.to_str().unwrap().to_owned())
    //             .collect::<Vec<String>>()
    //             .join("/");
    //         let found = re.replace_all(&found, "").to_string();
    //         format!("./{found}")
    //     } else {
    //         path
    //     }
    // }
    pub fn escape_reference(&self, ctx: Builder, ident: &ast::Identifier) -> Result {
        let ctx = match &ident.0 {
            a if a == "then" => ctx.put("['then']"),
            ident => ctx.put(".").put(ident.clone()),
        };
        Ok(ctx)
    }
    pub fn generate_member_segment<S>(
        &self,
        s: &S,
        ctx: Builder,
        elem: &ast::MemberSegment,
    ) -> Result
    where
        S: Visitor,
    {
        match elem {
            ast::MemberSegment::Computed(c) => {
                let ctx = ctx.put("[");
                let ctx = s.visit_expression(ctx, &c)?;
                Ok(ctx.put("]"))
            }
            ast::MemberSegment::Identifier(i) => self.escape_reference(ctx, i),
            ast::MemberSegment::Dispatch(i) => self.escape_reference(ctx, i),
        }
    }
    fn collect_targets(&self, expr: &ast::Destructuring, targets: &mut Vec<Identifier>) {
        for seg in expr.targets.iter() {
            match seg {
                ast::DestructuringSegment::Identifier(id) => targets.push(id.clone()),
                ast::DestructuringSegment::Destructuring((_, expr)) => {
                    self.collect_targets(expr, targets)
                }
            }
        }
    }
    pub fn gen_destruct(
        &self,
        ctx: Builder,
        source: &ast::Expression,
        expr: &ast::Destructuring,
    ) -> Result {
        let mut targets = Vec::<Identifier>::new();
        self.collect_targets(expr, &mut targets);
        let vars = targets
            .iter()
            .map(Identifier::by_name)
            .collect::<Vec<String>>()
            .join(", ");
        let ctx = ctx.line().put(format!("local {vars};"));
        let ctx = ctx.line().put("do").push();
        let ctx = ctx.line().put("local __destructure__ = ");
        let ctx = self.visit_expression(ctx, source)?.put(";");
        let ctx = self.gen("__destructure__".to_owned(), ctx, expr);
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }
    fn gen(&self, path: String, ctx: Builder, expr: &ast::Destructuring) -> Builder {
        match expr.origin {
            ast::DestructureOrigin::Tuple => {
                expr.targets
                    .iter()
                    .fold((ctx, 0u8), |(ctx, i), elem| match elem {
                        ast::DestructuringSegment::Identifier(id) => (
                            ctx.line().put(format!("{} = {}._{};", id.0, path, i)),
                            i + 1,
                        ),
                        ast::DestructuringSegment::Destructuring((_, dt)) => {
                            (self.gen(format!("{path}._{i}"), ctx, dt), i + 1)
                        }
                    })
                    .0
            }
            ast::DestructureOrigin::Array => {
                expr.targets
                    .iter()
                    .fold((ctx, 1u8), |(ctx, i), elem| match elem {
                        ast::DestructuringSegment::Identifier(id) => (
                            ctx.line().put(format!("{} = {}[{}];", id.0, path, i)),
                            i + 1,
                        ),
                        ast::DestructuringSegment::Destructuring((_, dt)) => {
                            (self.gen(format!("{path}[{i}]"), ctx, dt), i + 1)
                        }
                    })
                    .0
            }
            ast::DestructureOrigin::Table => {
                expr.targets.iter().fold(ctx, |ctx, elem| match elem {
                    ast::DestructuringSegment::Identifier(id) => {
                        ctx.line().put(format!("{} = {}.{};", id.0, path, id.0))
                    }
                    ast::DestructuringSegment::Destructuring((id, dt)) => {
                        self.gen(format!("{}.{}", path, id.0), ctx, dt)
                    }
                })
            }
        }
    }
}

impl Visitor for LuaEmitter {
    fn visit_macro_decorator(&self, ctx: Builder, stmt: &ast::MacroDecorator) -> Result {
        self.visit_statement(ctx, &stmt.target)
    }

    fn visit_return(&self, ctx: Builder, stmt: &ast::Return) -> Result {
        let ctx = ctx.line().put("return ");
        let ctx = self.visit_expression(ctx, &stmt.value)?;
        Ok(ctx.put(";"))
    }

    fn visit_block_expression(&self, ctx: Builder, expr: &ast::Do) -> Result {
        let ctx = ctx.put("(function()").push();
        let ctx = self.visit_block(ctx, &expr.body)?;
        let ctx = ctx.pop().unwrap().line().put("end)()");
        Ok(ctx)
    }

    fn visit_wrapped_expression(&self, ctx: Builder, expr: &ast::Expression) -> Result {
        let ctx = ctx.put("(");
        let ctx = self.visit_expression(ctx, expr)?;
        Ok(ctx.put(")"))
    }

    fn visit_identifier(&self, ctx: Builder, expr: &ast::Identifier) -> Result {
        Ok(ctx.put(expr.0.clone()))
    }

    fn visit_class(&self, ctx: Builder, stmt: &ast::Class) -> Result {
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
                "{}.prototype.__proto__ = {};",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ))
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
                    let is_self = if let Some(first) = f.arguments.first() {
                        first.name.0 == "self"
                    } else {
                        false
                    };
                    let level = if is_self { ".prototype." } else { "." }.to_string();
                    let ctx = ctx
                        .put(stmt.name.0.clone())
                        .put(level)
                        .put(f.name.0.clone())
                        .put(" = ");
                    let arguments = f.arguments.clone();
                    let ctx = self.visit_lambda(
                        ctx,
                        &ast::Lambda {
                            arguments,
                            body: ast::ScriptOrExpression::Script(f.body.clone()),
                        },
                    )?;
                    let ctx = ctx.put(";");
                    let ctx = f.decorators.iter().fold(Ok(ctx), |ctx, dec| {
                        let ctx = ctx?.line();
                        let ctx = self.visit_call(ctx, &dec.target)?;
                        let fn_ref = if is_self {
                            format!("{}.prototype.{}", stmt.name.0.clone(), f.name.0.clone())
                        } else {
                            format!("{}.{}", stmt.name.0.clone(), f.name.0.clone())
                        };
                        let ctx = ctx.put(format!(
                            "({}, \"{}\", {}, \"{}\", {{ is_static = {} }});",
                            fn_ref,
                            f.name.0.clone(),
                            stmt.name.0.clone(),
                            stmt.name.0.clone(),
                            !is_self
                        ));
                        Ok(ctx)
                    })?;
                    ctx
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
            };
            Ok(ctx)
        })?;
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

    fn visit_fn(&self, ctx: Builder, stmt: &ast::Function) -> Result {
        let ctx = ctx
            .line()
            .put("local function ")
            .put(stmt.name.0.clone())
            .put("(");
        let arg_names = stmt
            .arguments
            .iter()
            .map(|a| {
                if a.spread {
                    "...".into()
                } else {
                    a.name.0.clone()
                }
            })
            .collect::<Vec<String>>()
            .join(", ");
        let ctx = ctx.put(arg_names);
        let ctx = ctx.put(")").push();
        let ctx = if let Some(arg) = stmt.arguments.iter().last() {
            if arg.spread {
                ctx.line().put(format!("local {} = {{...}};", arg.name.0))
            } else {
                ctx
            }
        } else {
            ctx
        };
        let ctx = self.visit_block(ctx, &stmt.body)?;
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

    fn visit_assignment(&self, ctx: Builder, stmt: &ast::Assignment) -> Result {
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

    fn visit_declaration(&self, ctx: Builder, stmt: &ast::Let) -> Result {
        let ctx = match &stmt.target {
            ast::AssignmentTarget::Destructuring(e) => {
                self.gen_destruct(ctx, stmt.value.as_ref().unwrap(), e)?
            }
            ast::AssignmentTarget::Identifier(e) => {
                let ctx = ctx.line().put("local ").put(e.0.clone()).put(" = ");
                let ctx = self.visit_expression(ctx, stmt.value.as_ref().unwrap())?;
                ctx.put(";")
            }
        };
        Ok(ctx)
    }

    fn visit_expression_statement(&self, ctx: Builder, stmt: &ast::Expression) -> Result {
        Ok(self.visit_expression(ctx.line(), stmt)?.put(";"))
    }

    fn visit_lambda(&self, ctx: Builder, expr: &ast::Lambda) -> Result {
        let ctx = ctx.put("function(");
        let arg_src = expr
            .arguments
            .iter()
            .filter(|a| !a.spread)
            .map(|a| a.name.0.clone())
            .collect::<Vec<String>>();
        let spread = expr.arguments.iter().find(|a| a.spread);
        let ctx = ctx.put(arg_src.join(", "));
        let ctx = if spread.is_some() {
            if expr.arguments.len() > 1 {
                ctx.put(", ...")
            } else {
                ctx.put("...")
            }
        } else {
            ctx
        };
        let ctx = ctx.put(")").push();
        let ctx = if let Some(spread) = spread {
            ctx.line()
                .put(format!("local {} = {{...}};", spread.name.0))
        } else {
            ctx
        };
        let ctx = match &expr.body {
            ast::ScriptOrExpression::Script(e) => self.visit_block(ctx, e)?,
            ast::ScriptOrExpression::Expression(e) => self
                .visit_expression(ctx.line().put("return "), e)
                .map(|b| b.put(";"))?,
        };
        Ok(ctx.pop().unwrap().line().put("end"))
    }

    fn visit_reference(&self, ctx: Builder, expr: &ast::MemberExpression) -> Result {
        let ctx = self.visit_expression(ctx, &expr.head)?;
        let ctx = expr.tail.iter().fold(Ok(ctx), |ctx, elem| {
            self.generate_member_segment(self, ctx?, elem)
        })?;
        Ok(ctx)
    }

    fn visit_use_statement(&self, ctx: Builder, expr: &ast::UseStatement) -> Result {
        // let path = self.map_module_path(&expr.module);
        let path = expr.module.join(".");
        let tail = expr.module.last().unwrap();
        let ctx = if let Some(expand) = expr.expanded.as_ref() {
            let call = ast::CallSubExpression {
                callee: Some(ast::MemberExpression {
                    head: ast::Expression::Identifier(ast::Identifier("require".into())),
                    tail: vec![],
                }),
                arguments: vec![ast::Expression::String(ast::StringLiteral(path.clone()))],
            };
            let call = ast::CallExpression {
                head: call,
                tail: vec![],
            };
            let call = ast::Expression::Call(Box::new(call));
            self.gen_destruct(ctx, &call, expand)?
        } else {
            ctx.line()
                .put(format!("local {} = require(\"{}\");", tail, path))
        };
        Ok(ctx)
    }

    fn visit_call(&self, ctx: Builder, expr: &ast::CallExpression) -> Result {
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
                        ast::MemberSegment::Identifier(c) => ctx.put(".").put(c.0.clone()),
                        ast::MemberSegment::Dispatch(_) => {
                            panic!("Dispatch not allowed in this position")
                        }
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
                    ast::MemberSegment::Identifier(c) => ctx.put(".").put(c.0.clone()),
                    ast::MemberSegment::Dispatch(c) => ctx.put(":").put(c.0.clone()),
                }
            } else {
                ctx
            };
            ctx.put("(")
        } else {
            // TODO! Review when this branch is reached.
            ctx.put("(")
        };
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
                ast::MemberSegment::Identifier(i) => Ok(ctx?.put(".").put(i.0.clone())),
                ast::MemberSegment::Dispatch(i) => Ok(ctx?.put(":").put(i.0.clone())),
            },
        })?;
        Ok(ctx)
    }

    fn visit_tuple(&self, ctx: Builder, expr: &ast::Tuple) -> Result {
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

    fn visit_extern_block(&self, ctx: Builder, stmt: &ast::Extern) -> Result {
        match stmt.id.as_str() {
            "Lua" => {
                let ctx = ctx
                    .line()
                    .put("-- <extern \"Lua\"> --")
                    .put(&stmt.src)
                    .put("-- </extern> --");
                Ok(ctx)
            }
            _ => Ok(ctx),
        }
    }

    fn visit_number(&self, ctx: Builder, expr: &ast::Number) -> Result {
        let numeric_string = match expr {
            ast::Number::Float(e) => e.to_string(),
            ast::Number::Integer(e) => e.to_string(),
        };
        Ok(ctx.put(numeric_string))
    }

    fn visit_string(&self, ctx: Builder, expr: &ast::StringLiteral) -> Result {
        let ctx = ctx.put("\"").put(escape_string(expr.0.clone())).put("\"");
        Ok(ctx)
    }

    fn visit_unit(&self, ctx: Builder) -> Result {
        Ok(ctx.put("nil"))
    }

    fn visit_binary(&self, ctx: Builder, expr: &ast::BinaryExpression) -> Result {
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
            "++" => {
                // Native-to-native operator translation
                let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
                let ctx = ctx.put("..".to_owned());
                self.visit_expression(ctx.put(" "), &expr.right)
            }
            "<>" => {
                // Native-to-native operator translation
                let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
                let ctx = ctx.put("~=".to_owned());
                self.visit_expression(ctx.put(" "), &expr.right)
            }
            _ => {
                // Direct function translation
                let custom_fn_name = generate_operator_function_name(op.to_owned());
                let ctx = ctx.put(format!("{custom_fn_name}("));
                let ctx = self.visit_expression(ctx, &expr.left)?.put(", ");
                let ctx = self.visit_expression(ctx, &expr.right)?.put(")");
                Ok(ctx)
            }
        }
    }

    fn visit_unary(&self, ctx: Builder, expr: &ast::UnaryExpression) -> Result {
        let ctx = match expr.operator.clone().0.as_str() {
            "-" => ctx.put("-"),
            "not" => ctx.put("not "),
            "#?" => ctx.put("#"),
            op => todo!("Unary operator {:?} not supported!", op),
        };
        let ctx = self.visit_expression(ctx, &expr.expression)?;
        Ok(ctx)
    }

    fn visit_if(&self, ctx: Builder, expr: &ast::If) -> Result {
        let ctx = ctx.line().put("if ");
        let ctx = self.visit_expression(ctx, &expr.condition)?;
        let ctx = ctx.put(" then").push();
        let ctx = self.visit_block(ctx, &expr.body)?;
        let ctx = expr.branches.iter().fold(Ok(ctx), |ctx, (c, s)| {
            let ctx = ctx?.pop().unwrap().line().put("elseif ");
            let ctx = self.visit_expression(ctx, c)?;
            let ctx = ctx.put(" then").push();
            let ctx = self.visit_block(ctx, s)?;
            Ok(ctx)
        })?;
        let ctx = if let Some(eb) = expr.else_branch.as_ref() {
            let ctx = ctx.pop().unwrap().line().put("else").push();
            self.visit_block(ctx, eb)?
        } else {
            ctx
        };
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_table(&self, ctx: Builder, expr: &ast::Table) -> Result {
        let ctx = ctx.put("{");
        let ctx = if let Some((k, v)) = expr.key_values.first() {
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

    fn visit_vector(&self, ctx: Builder, expr: &ast::Vector) -> Result {
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

    fn visit_for(&self, ctx: Builder, expr: &ast::For) -> Result {
        let ctx = match &expr.handler {
            ast::AssignmentTarget::Destructuring(e) => {
                let ctx = ctx.line().put("for __destructured_iterator_target__ in ");
                let ctx = self.visit_expression(ctx, &expr.target)?;
                let ctx = ctx.put(" do").push();
                let dt = ast::Expression::Identifier(Identifier(
                    "__destructured_iterator_target__".into(),
                ));
                let ctx = self.gen_destruct(ctx, &dt, e)?;
                let ctx = self.visit_block(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
            ast::AssignmentTarget::Identifier(e) => {
                let ctx = ctx.line().put(format!("for {} in ", e.0.clone()));
                let ctx = self.visit_expression(ctx, &expr.target)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.visit_block(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
        };
        Ok(ctx)
    }

    fn visit_while(&self, ctx: Builder, expr: &ast::While) -> Result {
        let ctx = match &expr.condition {
            ast::ExpressionOrLet::Expression(e) => {
                let ctx = ctx.line().put("while ");
                let ctx = self.visit_expression(ctx, e)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.visit_block(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
            ast::ExpressionOrLet::Let(e) => {
                if let ast::AssignmentTarget::Identifier(id) = &e.target {
                    let ctx = ctx.line().put("do").push();
                    let ctx = self.visit_declaration(ctx, e)?;
                    let ctx = ctx.line().put(format!("while {} do", id.0.clone())).push();
                    let ctx = self.visit_block(ctx, &expr.body)?;
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

    fn visit_spread(&self, ctx: Builder, expr: &ast::SpreadExpression) -> Result {
        let ctx = ctx.put("__saturnus_spread__(");
        let ctx = self.visit_expression(ctx, &expr.expression)?;
        let ctx = ctx.put(")");
        Ok(ctx)
    }

    fn visit_loop(&self, ctx: Builder, expr: &ast::Loop) -> Result {
        let ctx = ctx.line().put("while true do").push();
        let ctx = self.visit_block(ctx, &expr.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_match(&self, _ctx: Builder, _expr: &ast::Match) -> Result {
        todo!("Match code generation not implemented yet.")
    }

    fn visit_script(&self, ctx: Builder, script: &crate::parser::Script) -> Result {
        let ctx = ctx
            .put("-- Generated by the Saturnus compiler 1.0")
            .line()
            .put("-- WARNING! Changes may be discarded at any moment!")
            .line()
            .put("local __saturnus_spread__ = table.unpack or unpack; -- Lua polyfill");
        self.visit_block(ctx, script)
    }
}
