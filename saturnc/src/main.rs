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
    backends::{LuaRt, RtEnv, Runtime},
    stdlib,
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
    let out = match target {
        CompileTarget::Lua => LuaCompiler::new().compile(source, options).unwrap(),
    };
    let mut out_file = File::create(&output).unwrap();
    write!(out_file, "{out}").unwrap();
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

fn run(input: PathBuf, options: CompilerOptions, dump_ir: bool) -> Result<(), ()> {
    let source_loc = input.clone().to_str().unwrap_or("").to_owned();
    let source = read_file_as_source(input).report_errors()?;
    let mut luac = LuaCompiler::new();
    let ir = luac.compile(source, options.clone()).report_errors()?;
    if dump_ir {
        println!("{}\n", format!("{ir}").dimmed());
    }
    let mut rt = LuaRt::default(RtEnv {
        globals: stdlib::init_native_modules(),
    });
    let stdlib_ir = luac
        .compile(
            CompilerSource {
                source: saturnus_rt::stdlib::SOURCE.to_owned(),
                location: Some(PathBuf::from("std")),
            },
            options.clone(),
        )
        .report_errors()?;
    rt.run(vec![(stdlib_ir, "stdlib".into()), (ir, source_loc)])
        .report_errors()?;
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
        Args::Run { input, dump_ir } => match run(input, options, dump_ir) {
            Ok(()) => (),
            Err(()) => exit(1),
        },
    }
}
