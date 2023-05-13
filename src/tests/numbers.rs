use super::compile_expr;

#[test]
fn positive_integer() {
    assert_eq!(compile_expr("1284"), "1284");
}

#[test]
fn negative_integer() {
    assert_eq!(compile_expr("-840"), "-840");
}

#[test]
fn positive_float() {
    assert_eq!(compile_expr("4.83"), "4.83");
}

#[test]
fn negative_float() {
    assert_eq!(compile_expr("-3.14"), "-3.14");
}
