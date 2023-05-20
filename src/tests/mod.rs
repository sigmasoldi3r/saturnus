use crate::{
    code::{Builder, Visitor},
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
    LuaEmitter
        .visit_script(Builder::new("  "), &Script::parse(input).unwrap())
        .unwrap()
        .collect()
}

pub fn compile_expr(input: &str) -> String {
    LuaEmitter
        .visit_expression(
            Builder::new("  "),
            &Script::parse_expression(input).unwrap(),
        )
        .unwrap()
        .collect()
}
