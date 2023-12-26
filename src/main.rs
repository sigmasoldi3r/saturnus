use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use code::info::InputFileInfo;
use errors::report_error;
use lua::{helpers::generate_module_chunk, visitor::LuaEmitter};
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
#[command(name = "Saturnus")]
#[command(version = "v0.2.0")]
#[command(author = "Pablo B. <pablobc.1995@gmail.com>")]
#[command(
    about = "Saturnus: A modern language that compiles to Lua",
    long_about = None // "Saturnus is a programming language that aims to have a simplified mix of Rust programming language and Lua"
)]
struct Args {
    #[arg(
        short,
        long,
        help = "An optional output file, if not provided the extension is replaced by .lua"
    )]
    output: Option<String>,
    #[arg(long, short)]
    verbose: bool,
    #[arg(help = "The input file to evaluate and/or compile")]
    input: String,
    #[arg(short, long, help = "Compiles the Saturnus script")]
    compile: bool,
    #[arg(
        short,
        long,
        help = "Specifies the output target runtime, only lua is supported for now."
    )]
    target: Option<String>,
    // This is now part of Janus
    // #[arg(
    //     short,
    //     long = "bin",
    //     help = "Compiles the Saturnus script into an executable binary"
    // )]
    // binary: bool,
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
    // Now std is inlined atop the entry point!
    // #[arg(long, help = "Inline the std library in each script")]
    // inline_std: bool,
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
    #[arg(long, help = "Extracts the STD library compiled")]
    extract_std_compiled: Option<PathBuf>,
    #[arg(long, help = "Extracts the STD library raw (Saturnus code)")]
    extract_std_raw: Option<PathBuf>,
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

const STD_SRC: &'static str = include_str!("assets/std.saturn");

fn precompile_std(compiler: &dyn Visitor) -> Result<(String, md5::Digest), RuntimeError> {
    // Precompile STD
    let std_src = STD_SRC;
    let std_src = Script::parse(std_src.to_owned()).unwrap();
    let std_src = compiler
        .visit_script(Builder::new("  "), &std_src)
        .unwrap()
        .collect();
    let crc = md5::compute(std_src.as_bytes());
    Ok((std_src, crc))
}

fn compile_main(
    script: &Script,
    compiler: &dyn Visitor,
    args: &Args,
) -> Result<String, RuntimeError> {
    let mut src = compiler
        .visit_script(Builder::new("  "), &script)
        .map_err(|err| RuntimeError::CompilationError(err))?
        .collect();
    if !args.no_std {
        let (std_src, _) = precompile_std(compiler)?;
        src = format!(
            "{}\n{}",
            generate_module_chunk(&"std".into(), &std_src),
            src
        );
    }
    Ok(src)
}

fn runtime_eval(script: &Script, compiler: &dyn Visitor, args: &Args) -> Result<(), RuntimeError> {
    let src = compile_main(script, compiler, args)?;
    let lua = rlua::Lua::new();
    lua.context(move |ctx| -> rlua::Result<()> {
        ctx.load(&src).eval()?;
        Ok(())
    })
    .map_err(|err| RuntimeError::EvaluationError(err))?;
    Ok(())
}

fn try_run(options: CompilationOptions, input: String, indent: String) -> Result<(), RuntimeError> {
    let compiler = lua::visitor::LuaEmitter::new(InputFileInfo {
        full_path: PathBuf::from(&options.in_path),
    });

    // We won't pop out that pesky "std.lua" file anymore!
    // if !options.args.no_std {
    //     let mut path = std::path::PathBuf::new();
    //     path.push(&options.out_path);
    //     path.pop();
    //     path.push("std.lua");
    //     let r = std::fs::read_to_string(&path).map(|r| md5::compute(r.as_bytes()));
    //     if r.is_err() || r.unwrap() != crc {
    //         std::fs::write(&path, std_src).unwrap();
    //     }
    // }

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
        if args.verbose {
            println!("Compiling {:?}...", in_path);
        }
        let output = compile_main(&script, &compiler, &args)?;
        if args.print {
            println!("\n------\n\n");
            std::io::stdout().write_all(output.as_bytes()).unwrap();
        } else {
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(output.as_bytes()).unwrap();
        }
    } else {
        runtime_eval(&script, &compiler, &args)?;
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

    {
        let compiler = lua::visitor::LuaEmitter::new(InputFileInfo {
            full_path: PathBuf::from("std.saturn"),
        });
        // Dump STD if provided
        if let Some(raw_std) = &args.extract_std_raw {
            std::fs::write(raw_std, STD_SRC).unwrap();
        }
        if let Some(compiled_std) = &args.extract_std_compiled {
            std::fs::write(compiled_std, precompile_std(&compiler).unwrap().0).unwrap();
        }
    }

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
                std::process::exit(-1);
            }
            RuntimeError::CompilationError(err) => {
                eprintln!("{:?}", err);
                std::process::exit(-1);
            }
        },
    }
}
