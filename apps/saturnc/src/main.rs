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
use saturnus::{Saturnus, Table, compiler::CompilerOptions, source::SourceCode};

fn read_file_as_source(
    mut input: PathBuf,
    override_path: Option<PathBuf>,
) -> Result<impl SourceCode, std::io::Error> {
    let mut source = String::new();
    File::open(&input)?.read_to_string(&mut source)?;
    input.set_extension("");
    struct Src {
        source: String,
        location: PathBuf,
    }
    impl SourceCode for Src {
        fn source(self) -> String {
            self.source
        }
        fn location(&self) -> Option<PathBuf> {
            Some(self.location.clone())
        }
    }
    Ok(Src {
        source,
        location: override_path.unwrap_or(input),
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
    let source = match read_file_as_source(input.clone(), options.override_mod_path.clone()) {
        Ok(val) => val,
        Err(err) => {
            panic!("Can't read {input:?} => {err}");
        }
    };
    let mut c = Saturnus::new();
    c.options = options.clone();
    let out = match target {
        CompileTarget::Lua => c.compile(source).unwrap(),
    };
    let mut out_file = File::create(&output).unwrap();
    write!(out_file, "{}", out.to_string()).unwrap();
}

fn produce_std(output: PathBuf) {
    let c = Saturnus::new();
    let out = c.compile(ststd::STDLIB_CODE).unwrap();
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
    // TODO ^^^^^^ Handle this.
    let source = read_file_as_source(input, options.override_mod_path.clone()).report_errors()?;
    let mut sat = Saturnus::new();
    sat.options = options.clone();
    let ir = sat.compile(source).report_errors()?;
    if dump_ir {
        println!("{}\n", format!("{}", ir.to_string()).dimmed());
    }
    struct InPlace {
        source: String,
        location: PathBuf,
    }
    impl SourceCode for InPlace {
        fn source(self) -> String {
            self.source
        }
        fn location(&self) -> Option<PathBuf> {
            Some(self.location.clone())
        }
    }
    // Load the cross platform stdlib
    sat.load(InPlace {
        source: ststd::STDLIB_CODE.to_owned(),
        location: PathBuf::from("std"),
    })
    .report_errors()?
    .exec()
    .report_errors()?;
    // Now Saturnus-Platform only functions.
    let globals = sat.globals();
    let mut __modules__: Table = globals.get("__modules__").unwrap().into();
    // Loading native libraries
    // TODO
    if !lib.is_empty() {
        eprintln!("WARNING: Native library loading is not implemented.");
    }
    globals.set("__modules__", __modules__).unwrap();
    sat.load_ir(ir).report_errors()?.exec().report_errors()?;
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
        Args::StdOutput { output, stdout } => {
            if stdout && output.is_some() {
                println!("Bad usage, --stdout and --output are mutually exclusive.");
            } else if !stdout && output.is_none() {
                println!("Specify one of --stdout or --output at least.");
            } else if stdout {
                struct OutLocal(String, PathBuf);
                impl SourceCode for OutLocal {
                    fn location(&self) -> Option<PathBuf> {
                        Some(self.1.clone())
                    }
                    fn source(self) -> String {
                        self.0
                    }
                }
                let out = Saturnus::new()
                    .compile(OutLocal(ststd::STDLIB_CODE.into(), PathBuf::from("std")))
                    .unwrap()
                    .to_string();
                println!("{out}");
            } else {
                produce_std(output.unwrap());
            }
        }
    }
}
