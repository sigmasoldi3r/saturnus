use crate::{code::Visitor, lua::LuaEmitter, parser::Script};

mod code;
mod lua;
mod parser;

fn main() {
    // let ast = Script::parse(include_str!("example.saturn"));
    // println!("{:?}", ast);
    let ast = Script::parse(include_str!("simple.saturn"));
    println!(
        "--- CODE ---\n{}\n--- ---- ---",
        LuaEmitter
            .visit_script(code::Builder::new("  "), &ast)
            .unwrap()
            .collect()
    );
}
