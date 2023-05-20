use super::compile_expr;
use spectral::prelude::*;

#[test]
fn bop_logic_and() {
    assert_that!(compile_expr("x and y")).is_equal_to("x and y".to_string());
}

#[test]
fn bop_logic_or() {
    assert_that!(compile_expr("x or y")).is_equal_to("x or y".to_string());
}
#[test]
fn bop_arithmetic() {
    assert_that!(compile_expr("x + y")).is_equal_to("x + y".to_string());
    assert_that!(compile_expr("x - y")).is_equal_to("x - y".to_string());
    assert_that!(compile_expr("x * y")).is_equal_to("x * y".to_string());
    assert_that!(compile_expr("x / y")).is_equal_to("x / y".to_string());
    assert_that!(compile_expr("x ** y")).is_equal_to("x ** y".to_string());
}
