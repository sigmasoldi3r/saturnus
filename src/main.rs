use crate::{ast_visitor::Walker, lua::LuaEmitter, parser::Script};

mod ast_visitor;
mod lua;
mod parser;

fn main() {
    // let ast = Script::parse(include_str!("example.foo"));
    // println!("{:?}", ast);
    let ast = Script::parse(
        "
@FooFighters
fn foo_fighters()
    class bar end
    the_foo2;
    return brotato(\"code\");
end
",
    );
    println!("{:?}", Walker(Box::new(LuaEmitter)).walk_script(ast));
}
