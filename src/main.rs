use crate::{code::Visitor, lua::LuaEmitter, parser::Script};

mod code;
mod lua;
mod parser;

fn main() {
    // let ast = Script::parse(include_str!("example.foo"));
    // println!("{:?}", ast);
    let ast = Script::parse(
        "
fn the_foo2()
    println(\"The foo was called, again.\");
end

fn brotato(what)
    return \"Brotato $what!\";
end

@FooFighters
fn foo_fighters()
    class bar end
    the_foo2();
    return brotato(\"code\");
end
",
    );
    println!(
        "--- CODE ---\n{}\n--- ---- ---",
        LuaEmitter
            .visit_script(code::Builder::new("  "), &ast)
            .unwrap()
            .collect()
    );
}
