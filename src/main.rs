#[cfg(test)]
#[macro_use]
extern crate spectral;

use crate::{code::Visitor, lua::LuaEmitter, parser::Script};

mod code;
mod lua;
mod parser;
#[cfg(test)]
mod tests;

fn main() {
    // let ast = Script::parse(include_str!("example.saturn"));
    // println!("{:?}", ast);
    let ast = Script::parse(include_str!("simple.saturn")).unwrap();
    println!(
        "--- CODE ---\n{}\n--- ---- ---",
        LuaEmitter
            .visit_script(code::Builder::new("  "), &ast)
            .unwrap()
            .collect()
    );
}
