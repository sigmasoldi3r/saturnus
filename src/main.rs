use std::{fs::File, io::Write, path::Path};

use clap::Parser;
use code::Visitor;
use errors::report_error;
use runtime::RuntimeError;

#[cfg(test)]
#[macro_use]
extern crate spectral;

mod code;
mod errors;
mod lua;
mod parser;
pub mod runtime;
#[cfg(test)]
mod tests;

#[derive(Parser, Clone)]
struct Args {
    #[arg(
        short,
        long,
        help = "An optional output file, if not provided the extension is replaced by .lua"
    )]
    output: Option<String>,
    #[arg(help = "The input file to evaluate and/or compile")]
    input: String,
    #[arg(short, long, help = "Compiles the Saturnus script")]
    compile: bool,
    #[arg(
        short = 'p',
        long = "print",
        help = "Prints the compilation result to the standard output"
    )]
    print: bool,
    #[arg(
        long,
        help = "If used, the compilation output emits tab characters. Ignores indentation parameter"
    )]
    use_tabs: bool,
    #[arg(
        default_value = "2",
        long,
        help = "The amount of space characters to use in each tab"
    )]
    indent: usize,
}

fn get_default_output(str: &Path) -> String {
    Path::new(str)
        .with_extension("lua")
        .to_str()
        .unwrap()
        .to_string()
}

struct CompilationOptions {
    args: Args,
    in_path: String,
    out_path: String,
}

fn try_run(options: CompilationOptions, input: String, indent: String) -> Result<(), RuntimeError> {
    let host = runtime::RuntimeHost::new(indent.clone());
    let script = parser::Script::parse(input).map_err(|err| RuntimeError::ParseError(err))?;

    let CompilationOptions {
        args,
        out_path,
        in_path,
    } = options;

    if args.compile {
        println!("Compiling {:?}...", in_path);
        let output = lua::LuaEmitter
            .visit_script(code::Builder::new(indent), &script)
            .map_err(|err| RuntimeError::CompilationError(err))?
            .collect();
        if args.print {
            println!("\n------\n\n");
            std::io::stdout().write_all(output.as_bytes()).unwrap();
        } else {
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(output.as_bytes()).unwrap();
        }
    } else {
        host.evaluate(&script)?;
    }

    Ok(())
}

fn main() {
    // Configure environment
    let args = Args::parse();
    let indent = if args.use_tabs {
        "\t".to_string()
    } else {
        " ".repeat(args.indent)
    };
    use std::fs::read_to_string;

    // Read input files
    let in_path = Path::new(&args.input);
    let out_path = args.clone().output.unwrap_or(get_default_output(in_path));
    let input = read_to_string(in_path).unwrap();

    let options = CompilationOptions {
        args: args.clone(),
        in_path: in_path.to_str().unwrap().to_owned(),
        out_path: out_path.to_owned(),
    };
    match try_run(options, input.clone(), indent.clone()) {
        Ok(_) => (),
        Err(err) => match err {
            RuntimeError::EvaluationError(err) => todo!(),
            RuntimeError::ParseError(err) => {
                let err = report_error(args.input.clone(), input.clone(), err);
                if args.compile && !args.print {
                    let mut out_file = File::create(out_path).unwrap();
                    let output = format!("error[=====[{}]=====]", err);
                    out_file.write_all(output.as_bytes()).unwrap();
                }
                eprintln!("{}\nCompilation failed", err);
            }
            RuntimeError::CompilationError(err) => todo!(),
        },
    }
}
