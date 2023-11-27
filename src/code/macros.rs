use std::collections::HashMap;

use crate::parser::ast;

use super::info::InputFileInfo;

struct MacroExpander;

impl MacroExpander {
    fn expand(call: &ast::CallExpression) -> ast::ScriptOrExpression {
        todo!()
    }
}

pub trait Macro {
    fn expand_call(&self, ast: &ast::CallExpression) -> ast::Expression;
}

struct PanicMacro(InputFileInfo);
impl Macro for PanicMacro {
    fn expand_call(&self, ast: &ast::CallExpression) -> ast::Expression {
        todo!("Panic macro not implemented yet.")
    }
}

struct FileMacro(InputFileInfo);
impl Macro for FileMacro {
    fn expand_call(&self, ast: &ast::CallExpression) -> ast::Expression {
        let value = self.0.full_path.as_os_str().to_string_lossy().to_string();
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
        macros.insert("file".into(), Box::new(PanicMacro(info.clone())));
        MacroHost { macros }
    }
}
