mod lua;

use crate::parsing::grammar;

#[test]
fn binary_expressions() {
    let parser = grammar::ExprParser::new();
    let out = parser.parse(r#"(((5+8/2 or "pootis" and 7>>9??10.5|>55))) + alberto(1+1, 2) + `poot 2+2` - `+`(1, 2)"#).unwrap();
    println!("--- BOP ---\n{out:?}");
}

#[test]
fn chained_call_expressions() {
    let parser = grammar::ExprParser::new();
    let out = parser
        .parse(r#"foo.bar() + bar.baz.bar().foo().bar.baz()()"#)
        .unwrap();
    println!("--- Call ---\n{out:?}");
}

#[test]
fn array_access_expressions() {
    let parser = grammar::ExprParser::new();
    let out = parser.parse(r#"foo.test?.[10, 20].lerp[10]"#).unwrap();
    println!("--- Array ---\n{out:?}");
}

#[test]
fn simple_program() {
    let parser = grammar::ProgramParser::new();
    let out = parser.parse(r#"foo + bar; yes(true);"#).unwrap();
    println!("--- Program ---\n{out:?}");
}

#[test]
fn simple_if() {
    let parser = grammar::ProgramParser::new();
    let out = parser
        .parse(r#"if a > b { foo(); } else if bar { foo(); } else { foo+1; }"#)
        .unwrap();
    println!("--- Simple If ---\n{out:?}");
}

#[test]
fn simple_let() {
    let parser = grammar::ProgramParser::new();
    let out = parser
        .parse(r#"let foo; let bar: Number; let x = 11;"#)
        .unwrap();
    println!("--- Simple Let ---\n{out:?}");
}

#[test]
fn simple_class() {
    let parser = grammar::ProgramParser::new();
    let out = parser
        .parse(r#"class Man { fn do() = true; fn be() { at(2+2); } }"#)
        .unwrap();
    println!("--- Simple Class ---\n{out:?}");
}

#[test]
fn lambda_expressions() {
    let parser = grammar::ProgramParser::new();
    let out = parser.parse(r#"{a, b => a + b}; {=>foo();};"#).unwrap();
    println!("--- Lambda Expression ---\n{out:?}");
}

#[test]
fn loops() {
    let parser = grammar::ProgramParser::new();
    let out = parser
        .parse(
            r#"
        loop {
            break;
        }
        for i in 1..10 {
            print("i = ${i}!");
        }
        for (k, v) in pairs(my_array) {
            print("k = ${k} -> v = ${v}");
        }
        while true {
            break;
        }
        "#,
        )
        .unwrap();
    println!("--- Loops ---\n{out:?}");
}
