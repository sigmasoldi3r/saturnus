use super::compile;
use spectral::prelude::*;

#[test]
fn test_for_loop() {
    let out = compile(
        "
      for chungus in [1, 2, 3] {
        print(chungus);
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
for chungus in {1, 2, 3} do
  print(chungus);
end"
        .to_string(),
    );
}

#[test]
fn test_ambiguous_syntax_for_loop() {
    let out = compile(
        "
      for chungus in ambiguity {
        print(chungus);
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
for chungus in ambiguity do
  print(chungus);
end"
        .to_string(),
    );
}

#[test]
fn test_while_loop() {
    let out = compile(
        "
      while chungus() {
        print(\"chungus!\");
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
while chungus() do
  print(\"chungus!\");
end"
        .to_string(),
    );
}

#[test]
fn test_while_with_let_loop() {
    let out = compile(
        "
      while let now = chungus() {
        print(now);
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
do
  local now = chungus();
  while now do
    print(now);
    now = chungus();
  end
end"
        .to_string(),
    );
}

#[test]
fn test_loop_loop() {
    let out = compile(
        "
      loop {
        print(chungus);
      }
    ",
    );
    assert_that!(out).is_equal_to(
        "
while true do
  print(chungus);
end"
        .to_string(),
    );
}
