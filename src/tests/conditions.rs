use super::compile;

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
    assert_eq!(
        out,
        "
if some.complex() > thing then
  print(\"hello world!\");
  local foo = function()
    return nil;
  end;
end"
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
    assert_eq!(
        out,
        "
if some.thing then
  thing();
else
  print(\"2 + 2 = \", 2 + 2);
end"
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
    assert_eq!(
        out,
        "
if some.thing then
  thing();
elseif the_branch() then
  print(\"2 + 2 = \", 2 + 2);
end"
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
    assert_eq!(
        out,
        "
if some.thing then
  thing();
elseif another.thing then
  another(one);
else
  print(\"2 + 2 = \", 2 + 2);
end"
    )
}
