use super::{compile, compile_expr};
use spectral::prelude::*;

#[test]
fn simple_named_function() {
    let out = compile(
        "
    fn foo(a, b) {
      return 2 + a + b;
    }
",
    );
    assert_that!(out).is_equal_to(
        "
local function foo(a, b)
  return 2 + a + b;
end"
        .to_string(),
    );
}

#[test]
fn simple_named_empty_function() {
    let out = compile(
        "
    fn foo(a, b) {}
",
    );
    assert_that!(out).is_equal_to(
        "
local function foo(a, b)
end"
        .to_string(),
    );
}

#[test]
fn inline_lambda() {
    let out = compile_expr("fn(a, b): a + b");
    assert_that!(out).is_equal_to(
        "function(a, b)
  return a + b;
end"
        .to_string(),
    );
}

#[test]
fn inline_empty_lambda() {
    let out = compile_expr("fn(a, b) {}");
    assert_that!(out).is_equal_to(
        "function(a, b)
end"
        .to_string(),
    );
}

#[test]
fn block_lambda() {
    let out = compile_expr(
        "
        fn(a, b) {
            return a + b;
        }
    "
        .trim(),
    );
    assert_that!(out).is_equal_to(
        "function(a, b)
  return a + b;
end"
        .to_string(),
    );
}

#[test]
fn nested_lambda_code() {
    let out = compile(
        "
    fn factory(a) {
        return fn(b) {
            return a + b;
        };
    }
    ",
    );
    assert_that!(out).is_equal_to(
        "
local function factory(a)
  return function(b)
    return a + b;
  end;
end"
        .to_string(),
    );
}
