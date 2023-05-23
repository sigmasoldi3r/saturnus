use std::{fs::File, io::Write, path::Path};

use clap::Parser;
use code::Visitor;
use console::style;

#[cfg(test)]
#[macro_use]
extern crate spectral;

mod code;
mod errors;
mod lua;
mod parser;
#[cfg(test)]
mod tests;

#[derive(Parser)]
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
    let out_path = args.output.unwrap_or(get_default_output(in_path));
    let input = read_to_string(in_path).unwrap();

    // Handle parsing errors
    match parser::Script::parse(input.clone()) {
        Ok(result) => {
            let output = lua::LuaEmitter
                .visit_script(
                    code::Builder::new(indent).put(
                        "-- Compiled by Saturnus compiler, warning: Changes may be discarded!",
                    ),
                    &result,
                )
                .unwrap()
                .collect();
            if args.compile {
                println!("Compiling {:?}...", in_path);
                if args.print {
                    println!("\n------\n\n");
                    std::io::stdout().write_all(output.as_bytes()).unwrap();
                } else {
                    let mut out_file = File::create(out_path).unwrap();
                    out_file.write_all(output.as_bytes()).unwrap();
                }
            } else {
                let rt = rlua::Lua::new();
                rt.context(move |ctx| -> rlua::Result<()> {
                    ctx.load(&output).eval()?;
                    Ok(())
                })
                .unwrap();
            }
        }
        Err(err) => {
            report_error(args.input.clone(), input.clone(), err);
            panic!("Compilation failed");
        }
    }
}

fn report_error(file: String, input: String, err: peg::error::ParseError<peg::str::LineCol>) {
    eprintln!("Failed to parse {} file!", file);
    let line = err.location.line;
    let col = err.location.column;
    let ep = err
        .expected
        .tokens()
        .map(|x| String::from(x))
        .reduce(|a, b| format!("{}, {}", a, b));
    eprintln!("At {}:{}:{}", file, line, col);
    if let Some(ep) = ep {
        eprintln!("Expected: one of {}", ep);
    }
    let lines = input.lines();
    let mut i = 0_usize;
    let mut pos = 0_usize;
    for line_str in lines {
        pos += 1;
        let to_sub = 3.min(line);
        if pos >= line - to_sub && pos < line + 5 {
            let n = pos.to_string();
            let numeric = format!("{:>4}", n);
            let numeric = style(numeric).blue();
            let divider = style("|").green().bold();
            eprintln!("{} {} {}", numeric, divider, line_str);
            if line == pos {
                let ted = line_str.len() - col;
                let premark = style("     |").red().bold();
                let spanner = format!(" {:2$}{:^<3$}", " ", "^", col - 2, ted);
                let spanner = style(spanner).red();
                let here = style("here").red();
                eprintln!("{} {} {here}", premark, spanner);
            }
            i += 1;
        }
        if i > 5 {
            break;
        }
    }
}
