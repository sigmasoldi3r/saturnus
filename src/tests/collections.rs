use spectral::prelude::*;

use super::compile_expr;

#[test]
fn simple_vector_expression() {
    let out = compile_expr("[1, 2, 3, (), \"hi\"]");
    assert_that!(out).is_equal_to("{1, 2, 3, nil, \"hi\"}".to_string());
}

#[test]
fn table_expression() {
    let out = compile_expr("{ rat, nest: 10, [2+2]: 4}");
    assert_that!(out).is_equal_to("{rat = rat, nest = 10, [2 + 2] = 4}".to_string());
}

#[test]
fn tuple_expressions() {
    let out = compile_expr("(1, 2, 3, \"yes\", { foo: \"bar\" })");
    assert_that!(out)
        .is_equal_to("{_0 = 1, _1 = 2, _2 = 3, _3 = \"yes\", _4 = {foo = \"bar\"}}".to_string());
}

#[test]
fn the_1_tuple_case() {
    let out = compile_expr("(45)");
    assert_that!(out).is_equal_to("(45)".to_string());
}

#[test]
fn the_0_tuple_case() {
    let out = compile_expr("()");
    assert_that!(out).is_equal_to("nil".to_string());
}
