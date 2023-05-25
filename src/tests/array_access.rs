use super::compile;
use spectral::prelude::*;

#[test]
fn simple_array_access_read() {
    let out = compile("foo = bar[baz];");
    assert_that!(out).is_equal_to(
        "
foo = bar[baz];"
            .to_string(),
    );
}

#[test]
fn simple_array_access_write() {
    let out = compile("foo.[bar] = baz;");
    assert_that!(out).is_equal_to(
        "
foo[bar] = baz;"
            .to_string(),
    );
}

#[test]
fn complex_mangled_array_access() {
    let out = compile("some.[complex].input = some.[foo].[2+2].bar;");
    assert_that!(out).is_equal_to(
        "
some[complex].input = some[foo][2 + 2].bar;"
            .to_string(),
    );
}
