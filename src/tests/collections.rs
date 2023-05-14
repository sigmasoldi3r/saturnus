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
