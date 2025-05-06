mod cli;
mod options;

use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::exit,
};

use cli::{Args, CompileTarget};
use colored::Colorize;
use options::OptionsAdapter;
use saturnus::compiling::{Compiler, CompilerOptions, CompilerSource, backends::LuaCompiler};
use saturnus_rt::{
    mem::St,
    native, table_get, table_set,
    vm::{StVm, types::Any},
};

fn read_file_as_source(mut input: PathBuf) -> Result<CompilerSource, std::io::Error> {
    let mut source = String::new();
    File::open(&input)?.read_to_string(&mut source)?;
    input.set_extension("");
    Ok(CompilerSource {
        source,
        location: Some(input),
    })
}

fn compile(
    input: PathBuf,
    options: CompilerOptions,
    target: CompileTarget,
    output: Option<PathBuf>,
) {
    let output = match output {
        Some(output) => output,
        None => {
            let mut input = input.clone();
            input.set_extension(target.ext());
            input
        }
    };
    let source = read_file_as_source(input).unwrap();
    let mut c = LuaCompiler::new();
    let out = match target {
        CompileTarget::Lua => c.compile(source, options).unwrap(),
    };
    let mut out_file = File::create(&output).unwrap();
    write!(out_file, "{}", out.to_string()).unwrap();
}

trait ErrorReporter<T> {
    fn report_errors(self) -> Result<T, ()>;
}
impl<T, E> ErrorReporter<T> for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn report_errors(self) -> Result<T, ()> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                eprintln!("{}", format!("{err:?}").red());
                Err(())
            }
        }
    }
}

async fn run(
    input: PathBuf,
    options: CompilerOptions,
    lib: Vec<PathBuf>,
    dump_ir: bool,
) -> Result<(), ()> {
    let source_loc = input.clone().to_str().unwrap_or("").to_owned();
    let source = read_file_as_source(input).report_errors()?;
    let mut luac = LuaCompiler::new();
    let ir = luac.compile(source, options.clone()).report_errors()?;
    if dump_ir {
        println!("{}\n", format!("{}", ir.to_string()).dimmed());
    }
    let vm = StVm::new();
    let stdlib_ir = luac
        .compile(
            CompilerSource {
                source: saturnstd::STDLIB_CODE.to_owned(),
                location: Some(PathBuf::from("std")),
            },
            options.clone(),
        )
        .report_errors()?;
    // Load the cross platform stdlib
    vm.load_program(stdlib_ir).exec().report_errors()?;
    // Now Saturnus-Platform only functions.
    let mut globals = vm.get_globals();
    let mut __modules__ = table_get!(vm; globals, "__modules__").unwrap_table();
    table_set!(vm; __modules__, "net" => native::net::load_mod(&vm));
    table_set!(vm; __modules__, "JSON" => native::json::load_mod(&vm));
    // Loading native libraries
    for lib_name in lib {
        println!("Loading native module {lib_name:?}...");
        unsafe {
            let lib = St::new(libloading::Library::new(lib_name.clone()).report_errors()?);
            let guard = lib.lock().await;
            let get_symbol_table: libloading::Symbol<extern "C" fn() -> (String, Vec<String>)> =
                guard.get(b"__saturnus_module_symbols__").report_errors()?;
            let (id, symbols) = get_symbol_table();
            let mut mod_table = vm.create_table().report_errors()?;
            for symbol in symbols {
                println!("Loading native wrapper {id}::{symbol}...");
                let wrapper = vm
                    .create_fn({
                        let lib = lib.clone();
                        let symbol = symbol.clone();
                        move |vm, args| {
                            let lib = lib.clone();
                            let symbol = symbol.clone();
                            async move {
                                let lib = lib.lock().await;
                                let func: libloading::Symbol<
                                    fn(StVm, Vec<Any>) -> saturnus_rt::vm::Result<Any>,
                                > = lib.get(symbol.as_bytes()).unwrap();
                                func(vm, args)
                            }
                        }
                    })
                    .report_errors()?;
                table_set!(vm; mod_table, symbol => wrapper);
            }
            table_set!(vm; __modules__, id => mod_table);
        }
    }
    table_set!(vm; globals, "__modules__" => __modules__);
    vm.load_program(ir).exec().report_errors()?;
    vm.process_pending().await;
    Ok(())
}

#[tokio::main]
async fn main() {
    use clap::Parser as _;
    let args = Args::parse();
    let adapter = OptionsAdapter::new();
    let options = adapter.parse_args(&args);
    match args {
        Args::Compile {
            input,
            target,
            output,
            ..
        } => compile(input, options, target, output),
        Args::Run {
            input,
            dump_ir,
            lib,
        } => {
            let task = tokio::spawn(run(input, options, lib.unwrap_or_default(), dump_ir));
            match task.await.unwrap() {
                Ok(()) => (),
                Err(()) => exit(1),
            }
        }
    }
}
