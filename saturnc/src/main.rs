mod cli;
mod options;

use std::{
    ffi::c_void,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::exit,
};

use cli::{Args, CompileTarget};
use colored::Colorize;
use options::OptionsAdapter;
use saturnus::compiling::{Compiler, CompilerOptions, CompilerSource, backends::LuaCompiler};
use saturnus_rt::{native, table_get, table_set, vm::StVm};

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
    vm.lock()
        .load_program(stdlib_ir)
        .exec()
        .await
        .report_errors()?;
    // Now Saturnus-Platform only functions.
    let mut globals = vm.lock().get_globals();
    let mut __modules__ = table_get!(vm; globals, "__modules__").unwrap_table();
    table_set!(vm; __modules__, "net" => native::net::load_mod(vm.clone()));
    table_set!(vm; __modules__, "JSON" => native::json::load_mod(vm.clone()));
    // Loading native libraries
    for lib in lib {
        println!("Loading native module {lib:?}...");
        unsafe {
            let lib = libloading::Library::new(lib).report_errors()?;
            let load_lib: libloading::Symbol<unsafe extern "stdcall" fn()> =
                lib.get(b"__load_saturnus_modules__").report_errors()?;
            //let vm_clone = vm.clone();
            //let ptr = std::mem::transmute(&vm_clone);
            // We get access_violation and I don't know why :/
            load_lib();
        }
    }
    table_set!(vm; globals, "__modules__" => __modules__);
    vm.lock().load_program(ir).exec().await.report_errors()?;
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
        } => match run(input, options, lib.unwrap_or_default(), dump_ir).await {
            Ok(()) => (),
            Err(()) => exit(1),
        },
    }
}
