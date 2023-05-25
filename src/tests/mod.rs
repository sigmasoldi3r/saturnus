use crate::{
    code::{Builder, Visitor},
    errors::report_error,
    lua::LuaEmitter,
    parser::Script,
};

mod array_access;
mod assignment;
mod binary_operators;
mod classes;
mod collections;
mod conditions;
mod functions;
mod loops;
mod numbers;

/// Fixture that streamlines the compilation, only for tests.
pub fn compile(input: &str) -> String {
    let result = match Script::parse(input) {
        Ok(ok) => ok,
        Err(err) => {
            report_error(file!().to_string(), input.to_string(), err);
            panic!("Test parsing failed");
        }
    };
    LuaEmitter
        .visit_script(Builder::new("  "), &result)
        .unwrap()
        .collect()
}

pub fn compile_expr(input: &str) -> String {
    let result = match Script::parse_expression(input) {
        Ok(ok) => ok,
        Err(err) => {
            report_error(file!().to_string(), input.to_string(), err);
            panic!("Test parsing failed");
        }
    };
    LuaEmitter
        .visit_expression(Builder::new("  "), &result)
        .unwrap()
        .collect()
}
