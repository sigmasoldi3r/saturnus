use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use clap::Parser;
use code::Visitor;

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
    #[arg(short, long, help = "The input file")]
    input: String,
    #[arg(
        short = 'p',
        long = "print",
        help = "Prints the compilation result to the standard output"
    )]
    pipe: bool,
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
    indentation: usize,
}

fn get_default_output(str: &Path) -> String {
    Path::new(str)
        .with_extension("lua")
        .to_str()
        .unwrap()
        .to_string()
}

fn main() {
    let args = Args::parse();
    let indent = if args.use_tabs {
        "\t".to_string()
    } else {
        " ".repeat(args.indentation)
    };
    use std::fs::read_to_string;
    let in_path = Path::new(&args.input);
    println!("Compiling {:?}...", in_path);
    let out_path = args.output.unwrap_or(get_default_output(in_path));
    let input = read_to_string(in_path).unwrap();
    let output = lua::LuaEmitter
        .visit_script(
            code::Builder::new(indent),
            &parser::Script::parse(input).unwrap(),
        )
        .unwrap()
        .collect();
    let mut out_file = File::create(out_path).unwrap();
    out_file.write_all(output.as_bytes()).unwrap();
}
