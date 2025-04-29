mod cli;
mod options;

use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use cli::{Args, CompileTarget};
use options::OptionsAdapter;
use saturnus::compiling::{Compiler, CompilerOptions, CompilerSource, backends::LuaCompiler};
use saturnus_rt::{
    backends::{LuaRt, RtEnv, Runtime},
    core::Table,
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

fn run(input: PathBuf, options: CompilerOptions, dump_ir: bool) {
    let source = read_file_as_source(input).unwrap();
    let ir = LuaCompiler::new().compile(source, options).unwrap();
    if dump_ir {
        println!("{ir}");
    }
    let mut rt = LuaRt::default(RtEnv {
        globals: Table::new(),
    });
    rt.run(ir).unwrap();
}

fn main() {
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
        Args::Run { input, dump_ir } => run(input, options, dump_ir),
    }
}
