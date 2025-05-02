use std::path::PathBuf;

use crate::compiling::{Compiler, CompilerOptions, CompilerSource, backends::LuaCompiler};

#[test]
fn test_simple_use() {
    let mut c = LuaCompiler::new();
    let source = CompilerSource {
        location: Some(PathBuf::from("foo/bar/module")),
        source: r#"// Simple test
        use std::{ ops::{ `|>`, `..` }, string::utils, math };
        
        pub fn exported_func() = 2 + 2;
        "#
        .into(),
    };
    let options = CompilerOptions {
        unit_interop: true,
        ..Default::default()
    };
    let ir = c.compile(source, options).unwrap();
    println!("---- MODLIB Output ----\n{}", ir.to_string());
}
