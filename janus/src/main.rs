mod compilation;
mod deps;
mod dir;
mod display;
mod errors;
mod janusfile;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use compilation::CompilationError;

use crate::{compilation::CompilationHost, errors::ExitCode, janusfile::JanusWorkspaceConfig};

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
    /// Cleans the build cache only
    Clean,
}

fn handle_compilation_error(err: CompilationError) {
    match err {}
}

fn process_build(args: Args) {
    if let Some(workspace) = JanusWorkspaceConfig::parse_janus_file(&args.path) {
        let JanusWorkspaceConfig {
            project_type,
            project,
            build,
            dependencies,
        }: JanusWorkspaceConfig = workspace;
        let dependencies = dependencies.unwrap_or_default();
        let build = build.unwrap_or_default();
        let project = project.unwrap_or_default();
        let result = match project_type.as_str() {
            "lib" => CompilationHost::new().compile(
                compilation::CompilationMode::Lib,
                dependencies,
                build,
                project,
            ),
            "bin" => CompilationHost::new().compile(
                compilation::CompilationMode::Bin,
                dependencies,
                build,
                project,
            ),
            _ => {
                eprintln!("Invalid project type {}!", project_type);
                Ok(())
            }
        };
        match result {
            Ok(_) => println!("\nOk - project compiled"),
            Err(err) => handle_compilation_error(err),
        }
    } else {
        eprintln!(
            "Could not parse the janus file! Check the docs to see the correct format and fields."
        );
        ExitCode::BadJanusFile.exit();
    }
}

fn main() {
    let args = Args::parse();
    match args.order {
        Order::Build => process_build(args),
        Order::Run => todo!("Not done"),
        Order::Init => todo!("Not done"),
        Order::Clean => {
            let info = JanusWorkspaceConfig::parse_janus_file(&args.path).unwrap();
            std::fs::remove_dir_all(
                info.build
                    .unwrap_or_default()
                    .output
                    .unwrap_or("dist".into())
                    .join("cache"),
            )
            .unwrap();
        }
    }
}
