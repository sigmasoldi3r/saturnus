use super::{compile, compile_expr};
use spectral::prelude::*;

#[test]
fn simple_named_function() {
    let out = compile(
        "
    fn foo(a, b)
      return 2 + a + b;
    end
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
    fn foo(a, b) end
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
    let out = compile_expr("fn(a, b) a + b end");
    assert_that!(out).is_equal_to(
        "function(a, b)
  return a + b;
end"
        .to_string(),
    );
}

#[test]
fn inline_empty_lambda() {
    let out = compile_expr("fn(a, b) end");
    assert_that!(out).is_equal_to(
        "function(a, b)
end"
        .to_string(),
    );
}

#[test]
fn block_lambda() {
    let out = compile_expr(
        "fn(a, b)
      return a + b;
    end",
    );
    assert_that!(out).is_equal_to(
        "function(a, b)
  return a + b;
end"
        .to_string(),
    );
}
