use super::compile_expr;
use spectral::prelude::*;

#[test]
fn positive_integer() {
    assert_that!(compile_expr("1284")).is_equal_to("1284".to_string());
}

#[test]
fn negative_integer() {
    assert_that!(compile_expr("-840")).is_equal_to("-840".to_string());
}

#[test]
fn positive_float() {
    assert_that!(compile_expr("4.83")).is_equal_to("4.83".to_string());
}

#[test]
fn negative_float() {
    assert_that!(compile_expr("-3.14")).is_equal_to("-3.14".to_string());
}
