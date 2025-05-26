use crate::Saturnus;

#[test]
fn test_strings() {
    let raw = r#" let x = "This is a \n\"escaped\"\n string!"; "#;
    let c = Saturnus::new();
    let out = c.compile(raw).unwrap();
    let plain_ir = out.to_string();
    let plain_ir = plain_ir.trim().to_owned();
    let plain_ir = plain_ir.split("\n").last().unwrap();
    assert_eq!(
        plain_ir,
        r#"local x = "This is a \n\"escaped\"\n string!";"#
    )
}

#[test]
fn test_ml_strings() {
    let raw = r#" let x = "In saturnus
Strings should be OK
to be just

Multilne :)"; "#;
    let c = Saturnus::new();
    let out = c.compile(raw).unwrap();
    let plain_ir = out.to_string();
    let plain_ir = plain_ir.trim().to_owned();
    let plain_ir = plain_ir
        .split("\n")
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        plain_ir,
        r#"local x = [[In saturnus\nStrings should be OK\nto be just\n\nMultilne :)]];"#
    )
}
