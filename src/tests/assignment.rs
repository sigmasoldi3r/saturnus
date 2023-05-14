use super::compile;
use spectral::prelude::*;

#[test]
fn simple_assignment() {
    let out = compile("foo = 8;");
    assert_that!(out).is_equal_to(
        "
foo = 8;"
            .to_string(),
    );
}

#[test]
fn complex_assignment() {
    let out = compile("foo += 8;");
    assert_that!(out).is_equal_to(
        "
foo = foo + 8;"
            .to_string(),
    );
}
