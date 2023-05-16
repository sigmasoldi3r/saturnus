use super::compile;
use spectral::prelude::*;

#[test]
fn simple_if_statement() {
    let out = compile(
        "
      if some.complex() > thing {
        print(\"hello world!\");
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
if some:complex() > thing then
  print(\"hello world!\");
end"
        .to_string(),
    )
}

#[test]
fn if_with_else() {
    let out = compile(
        "
  if some.thing {
    thing();
  } else {
    print(\"2 + 2 = \", 2 + 2);
  }
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
  if some.thing {
    thing();
  } else if the_branch() {
    print(\"2 + 2 = \", 2 + 2);
  }
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
  if some.thing {
    thing();
  } else if another.thing {
    another(one);
  } else {
    print(\"2 + 2 = \", 2 + 2);
  }
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
