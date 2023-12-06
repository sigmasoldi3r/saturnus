use std::collections::HashMap;

use crate::parser::ast::{
    self, CallExpression, CallSubExpression, Expression, Identifier, MemberExpression,
    StringLiteral,
};

use super::info::InputFileInfo;

// struct MacroExpander;

// impl MacroExpander {
//     fn expand(call: &ast::CallExpression) -> ast::ScriptOrExpression {
//         todo!()
//     }
// }

pub trait Macro {
    fn expand_call(&self, ast: &ast::MacroCallExpression) -> ast::Expression;
}

struct PanicMacro(InputFileInfo);
impl Macro for PanicMacro {
    fn expand_call(&self, ast: &ast::MacroCallExpression) -> ast::Expression {
        Expression::Call(Box::new(CallExpression {
            head: CallSubExpression {
                callee: Some(MemberExpression {
                    head: Expression::Identifier(Identifier("error".to_string())),
                    tail: vec![],
                }),
                arguments: ast.arguments.clone().unwrap_or(vec![]),
            },
            tail: vec![],
        }))
    }
}

struct IncludeTextMacro;
impl Macro for IncludeTextMacro {
    fn expand_call(&self, ast: &ast::MacroCallExpression) -> ast::Expression {
        if let Some(args) = &ast.arguments {
            if let Some(arg) = args.first() {
                if let Expression::String(value) = arg {
                    if value.prefix.is_some() {
                        panic!("include_text!() string argument cannot have prefix!");
                    }
                    let value = std::fs::read_to_string(&value.value).unwrap();
                    let value = value.replace("\"", "\\\"");
                    return Expression::String(StringLiteral {
                        prefix: None,
                        value,
                    });
                }
            }
        }
        panic!("include_text!() macro needs to be called with a constant string argument!");
    }
}

struct IncludeBytesMacro;
impl Macro for IncludeBytesMacro {
    fn expand_call(&self, ast: &ast::MacroCallExpression) -> ast::Expression {
        if let Some(args) = &ast.arguments {
            if let Some(arg) = args.first() {
                if let ast::Expression::String(value) = arg {
                    if value.prefix.is_some() {
                        panic!("include_bytes!() string argument cannot have prefix!");
                    }
                    let value = std::fs::read(&value.value).unwrap();
                    let expressions = value
                        .iter()
                        .map(|int| {
                            ast::Expression::Number(ast::Number {
                                value: ast::NumberVariant::Hexadecimal(int.clone() as i64),
                                postfix: None,
                            })
                        })
                        .collect::<Vec<ast::Expression>>();
                    return ast::Expression::Vector(ast::Vector { expressions });
                }
            }
        }
        panic!("include_bytes!() macro needs to be called with a constant string argument!");
    }
}

struct IncludeBase64Macro;
impl Macro for IncludeBase64Macro {
    fn expand_call(&self, ast: &ast::MacroCallExpression) -> ast::Expression {
        if let Some(args) = &ast.arguments {
            if let Some(arg) = args.first() {
                if let ast::Expression::String(value) = arg {
                    use base64::Engine;
                    if value.prefix.is_some() {
                        panic!("include_base64!() string argument cannot have prefix!");
                    }
                    let value = std::fs::read(&value.value).unwrap();
                    let value = base64::engine::general_purpose::STANDARD.encode(value);
                    return Expression::String(StringLiteral {
                        prefix: None,
                        value,
                    });
                }
            }
        }
        panic!("include_base64!() macro needs to be called with a constant string argument!");
    }
}

struct FileMacro(InputFileInfo);
impl Macro for FileMacro {
    fn expand_call(&self, _: &ast::MacroCallExpression) -> ast::Expression {
        let value = self.0.full_path.as_os_str().to_string_lossy().to_string();
        let value = value.replace("\\", "/");
        ast::Expression::String(ast::StringLiteral {
            value,
            prefix: None,
        })
    }
}

pub struct MacroHost {
    pub macros: HashMap<String, Box<dyn Macro>>,
}
impl MacroHost {
    pub fn new(info: InputFileInfo) -> Self {
        let mut macros: HashMap<String, Box<dyn Macro>> = HashMap::new();
        macros.insert("panic".into(), Box::new(PanicMacro(info.clone())));
        macros.insert("file".into(), Box::new(FileMacro(info.clone())));
        macros.insert("include_str".into(), Box::new(IncludeTextMacro));
        macros.insert("include_bytes".into(), Box::new(IncludeBytesMacro));
        macros.insert("include_base64".into(), Box::new(IncludeBase64Macro));
        MacroHost { macros }
    }
}
