use std::path::PathBuf;

use saturnus_rt::{
    backends::{LuaRt, RtEnv, Runtime},
    core::{Any, Callable, IntoSaturnus, Table},
};

use crate::compiling::{Compiler, CompilerOptions, CompilerSource, backends::LuaCompiler};

#[test]
fn test_stdlib_output() {
    let mut c = LuaCompiler::new();
    let source = CompilerSource {
        location: Some(PathBuf::from("std")),
        source: saturnus_rt::stdlib::SOURCE.to_string(),
    };
    let options = CompilerOptions {
        unit_interop: true,
        ..Default::default()
    };
    let out = c.compile(source, options).unwrap();
    println!("---- STDLIB Output ----\n{out}");
}

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
    let out = c.compile(source, options).unwrap();
    println!("---- MODLIB Output ----\n{out}");
}

#[test]
fn test_simple_program() {
    let mut c = LuaCompiler::new();
    let source = CompilerSource {
        location: None,
        source: r#"
        fn map(mapper) = {
            let out = [];
            for (k, v) in ipairs(it) {
                out[k] = mapper(v, k);
            }
            return out;
        };

        fn `|>`(l, r) = r(l);

        let d = [1, 2, 3] |> map.{ it + 1 };

        class ArrayPrint {
            static fn print(data) = print("[" ++ table::concat(data, ", ") ++ "]");
        }

        let r = std::test(1, 2, "foo", true);
        print("r = " ++ tostring(r));
        print("Running saturnus v" ++ std.version);
        ArrayPrint::print(d);
        "#
        .into(),
    };
    let options = CompilerOptions {
        unit_interop: true,
        ..Default::default()
    };
    let mut globals = Table::new();
    let mut std_lib = Table::new();
    std_lib.set("version", 1.5);
    std_lib.set(
        "test",
        Callable::new(|args: Vec<Any>| {
            println!("Foo bar test man! args = {args:?}");
            let Any::Integer(a) = args[0] else { panic!() };
            let Any::Integer(b) = args[1] else { panic!() };
            format!("What the foo? {a} + {b} = {}", a + b).into_saturnus()
        }),
    );
    globals.set("std", std_lib);
    let mut rt = LuaRt::default(RtEnv { globals });
    let out = c.compile(source, options).unwrap();
    let out = rt.run(vec![(out, String::new())]).unwrap();
    println!("---- Lua Output ----\n{out:?}");
}
