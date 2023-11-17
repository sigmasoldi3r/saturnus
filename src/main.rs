use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use errors::report_error;
use parser::Script;
use runtime::RuntimeError;

use crate::code::{ast_visitor::Visitor, builder::Builder};

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
    #[arg(long, help = "Skips the std library")]
    no_std: bool,
    #[arg(long, help = "Inline the std library in each script")]
    inline_std: bool,
    #[arg(
        long,
        help = "Outputs the saturnus code to stdout preprocessed but without compiling"
    )]
    dump_saturnus: bool,
    #[arg(
        short = 'm',
        long = "mod",
        help = "Additional module root paths to load"
    )]
    modules: Vec<String>,
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

// fn scrap_modules(map: &mut HashMap<String, PathBuf>, roots: &Vec<String>) {
//     let re = regex::Regex::new(r"\.saturn$").unwrap();
//     for root in roots.iter() {
//         for entry in glob::glob(format!("{}/**/*.saturn", root).as_str()).unwrap() {
//             let entry: PathBuf = entry.unwrap();
//             let mod_name = entry
//                 .iter()
//                 .skip(1)
//                 .map(|p| p.to_str().unwrap().to_owned())
//                 .collect::<Vec<String>>()
//                 .join(".");
//             let mod_name = re.replace_all(mod_name.as_str(), "");
//             map.insert(mod_name.to_string(), entry);
//         }
//     }
// }

fn try_run(options: CompilationOptions, input: String, indent: String) -> Result<(), RuntimeError> {
    let compiler = lua::visitor::LuaEmitter::new();
    // Precompile STD
    let std_src = include_str!("assets/std.saturn");
    let std_src = Script::parse(std_src.to_owned()).unwrap();
    let std_src = compiler
        .visit_script(Builder::new("  "), &std_src)
        .unwrap()
        .collect();
    let crc = md5::compute(std_src.as_bytes());

    if !options.args.no_std {
        let mut path = std::path::PathBuf::new();
        path.push(&options.out_path);
        path.pop();
        path.push("std.lua");
        let r = std::fs::read_to_string(&path).map(|r| md5::compute(r.as_bytes()));
        if r.is_err() || r.unwrap() != crc {
            std::fs::write(&path, std_src).unwrap();
        }
    }

    // scrap_modules(&mut compiler.module_mapping, &options.args.modules);

    if options.args.dump_saturnus {
        println!("{input}");
        return Ok(());
    }

    let script = parser::Script::parse(input).map_err(|err| RuntimeError::ParseError(err))?;

    let CompilationOptions {
        args,
        out_path,
        in_path,
    } = options;

    if args.compile {
        println!("Compiling {:?}...", in_path);
        let output = compiler
            .visit_script(Builder::new(indent), &script)
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
        let host: runtime::RuntimeHost =
            runtime::RuntimeHost::new(indent.clone(), Box::new(compiler));
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
            RuntimeError::EvaluationError(err) => eprintln!("{}", err),
            RuntimeError::ParseError(err) => {
                let err = report_error(args.input.clone(), input.clone(), err);
                if args.compile && !args.print {
                    let mut out_file = File::create(out_path).unwrap();
                    let output = format!("error[=====[{}]=====]", err);
                    out_file.write_all(output.as_bytes()).unwrap();
                }
                eprintln!("{}\nCompilation failed", err);
            }
            RuntimeError::CompilationError(err) => eprintln!("{:?}", err),
        },
    }
}
