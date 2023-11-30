mod compilation;
mod janusfile;
mod jobs;

use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use compilation::CompilationError;
use console::style;
use glob::glob;
use indicatif::ProgressBar;
use std::process::Command;

use crate::{
    compilation::{BinaryCompiler, LibraryCompiler},
    janusfile::JanusWorkspaceConfig,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    order: Order,
    #[arg(
        long,
        short,
        help = "Optionally, specify a path to a Janus.toml folder"
    )]
    path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Order {
    /// Builds the current project according to Janus.toml file
    Build,
    /// Initializes a new empty Saturnus project
    Init,
    /// Runs the current project, or the examples if library
    Run,
}

fn handle_compilation_error(err: CompilationError) {
    match err {}
}

fn process_build(args: Args) {
    if let Some(workspace) = JanusWorkspaceConfig::parse_janus_file(&args.path) {
        let JanusWorkspaceConfig {
            project_type,
            project: _,
            build,
        } = workspace;
        let build = build.unwrap_or_default();
        let result = match project_type.as_str() {
            "lib" => LibraryCompiler::new().compile(build),
            "bin" => BinaryCompiler::new().compile(build),
            _ => {
                eprintln!("Invalid project type {}!", project_type);
                Ok(())
            }
        };
        match result {
            Ok(_) => println!("Ok - project compiled"),
            Err(err) => handle_compilation_error(err),
        }
    } else {
        eprintln!(
            "Could not parse the janus file! Check the docs to see the correct format and fields."
        );
    }
    // println!("{} Collecting sources...", style("[2/4]").bold());
    // let sources = collect_sources()?;
    // // In the future this process will be parallelized
    // println!("{} Compiling sources...", style("[3/4]").bold());
    // let pb = ProgressBar::new(sources.len() as u64);
    // for source in sources {
    //     let result = Command::new("saturnus")
    //         .args(vec!["-c", source.as_str()])
    //         .output();
    //     if result.is_err() {
    //         eprintln!(
    //             "\n{} Failed to compile {}!",
    //             style("FATAL:").bold().red(),
    //             style(source.clone()).bold()
    //         );
    //     }
    //     let output = result.unwrap();
    //     if !output.status.success() {
    //         eprintln!(
    //             "\nFailed to compile {}!\n{}\n{} {}",
    //             style(source.clone()).bold().underlined(),
    //             style(String::from_utf8(output.stdout).unwrap())
    //                 .dim()
    //                 .color256(8_u8),
    //             style("FATAL:").bold().red(),
    //             String::from_utf8(output.stderr).unwrap()
    //         );
    //         return None;
    //     }
    //     pb.inc(1);
    // }
}

fn main() {
    let args = Args::parse();
    match args.order {
        Order::Build => process_build(args),
        Order::Run => todo!("Not done"),
        Order::Init => todo!("Not done"),
    }
}
