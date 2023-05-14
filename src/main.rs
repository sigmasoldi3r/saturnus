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
}

fn get_default_output(str: &String) -> String {
    Path::new(str)
        .with_extension("lua")
        .to_str()
        .unwrap()
        .to_string()
}

fn main() {
    let args = Args::parse();
    let in_path = args.input;
    let out_path = args.output.unwrap_or(get_default_output(&in_path));
    let mut in_file = File::open(in_path).unwrap();
    let mut input = String::new();
    in_file.read_to_string(&mut input).unwrap();
    let output = lua::LuaEmitter
        .visit_script(
            code::Builder::new("  "),
            &parser::Script::parse(input).unwrap(),
        )
        .unwrap()
        .collect();
    let mut out_file = File::create(out_path).unwrap();
    out_file.write_all(output.as_bytes()).unwrap();
}
