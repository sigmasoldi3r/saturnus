use super::compile;
use spectral::prelude::*;

#[test]
fn simple_if_statement() {
    let out = compile(
        "
      if some.complex() > thing then
        print(\"hello world!\");
        let foo = fn() () end;
      end
    ",
    );
    assert_that!(out).is_equal_to(
        "
if some.complex() > thing then
  print(\"hello world!\");
  local foo = function()
    return nil;
  end;
end"
        .to_string(),
    )
}

#[test]
fn if_with_else() {
    let out = compile(
        "
  if some.thing then
    thing();
  else
    print(\"2 + 2 = \", 2 + 2);
  end
",
    );
    assert_that!(out).is_equal_to(
        "
if some.thing then
  thing();
else
  print(\"2 + 2 = \", 2 + 2);
end"
        .to_string(),
    )
}

#[test]
fn if_with_branches() {
    let out = compile(
        "
  if some.thing then
    thing();
  else if the_branch() then
    print(\"2 + 2 = \", 2 + 2);
  end
",
    );
    assert_that!(out).is_equal_to(
        "
if some.thing then
  thing();
elseif the_branch() then
  print(\"2 + 2 = \", 2 + 2);
end"
        .to_string(),
    )
}

#[test]
fn if_with_branches_and_else() {
    let out = compile(
        "
  if some.thing then
    thing();
  else if another.thing then
    another(one);
  else
    print(\"2 + 2 = \", 2 + 2);
  end
",
    );
    assert_that!(out).is_equal_to(
        "
if some.thing then
  thing();
elseif another.thing then
  another(one);
else
  print(\"2 + 2 = \", 2 + 2);
end"
        .to_string(),
    )
}
