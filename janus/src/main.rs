mod compilation;
mod deps;
mod dir;
mod display;
mod errors;
mod janusfile;

use std::{collections::HashMap, path::PathBuf};

use clap::{Parser, Subcommand};
use compilation::CompilationError;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};

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

fn init_project() {
    let cwd = std::env::current_dir().unwrap();
    let default_name = cwd.file_name().unwrap().to_str().unwrap().to_string();
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name of the project")
        .default(default_name)
        .interact_text()
        .unwrap();
    let janus = janusfile::JanusWorkspaceConfig {
        project_type: "bin".into(),
        project: Some(janusfile::JanusProject {
            name: Some(name.clone()),
            description: Some(format!("{name} project")),
            author: None,
            version: Some("1.0.0".to_string()),
        }),
        build: Some(janusfile::JanusBuild {
            source: None,
            output: None,
            main: None,
            format: Some("bin".to_string()),
            target: None,
            module_system: None,
            no_std: None,
            modules: None,
        }),
        dependencies: Some(HashMap::new()),
    };
    let out = toml::to_string_pretty(&janus).unwrap();
    if let Err(e) = std::fs::write("Janus.toml", out) {
        eprintln!(
            "{}\n{:?}",
            style("Error! Could not create the Janus.toml file!").red(),
            e
        );
    }
    if let Err(_) = std::fs::write(".gitignore", "/dist\n*.lua") {
        eprintln!(
            "{}",
            style("Could not create the .gitignore file")
                .yellow()
                .dim()
                .italic()
        );
    }
    if let Err(e) = std::fs::create_dir("src") {
        eprintln!(
            "{}\n{:?}",
            style("Error! Could not create the src/ directory!").red(),
            e
        );
    }
    if let Err(e) = std::fs::write(
        "src/main.saturn",
        "// See https://github.com/sigmasoldi3r/Saturnus#saturnus\nprint(\"Hello World!\");",
    ) {
        eprintln!(
            "{}\n{:?}",
            style("Error! Could not create the main example file at src/main.saturn!").red(),
            e
        );
    }
    if let Err(_) = std::process::Command::new("git")
        .arg("init")
        .arg("-b")
        .arg("main")
        .output()
    {
        eprintln!(
            "{}",
            style("Could not initialize the git repository")
                .color256(8_u8)
                .dim()
                .italic()
        );
    }
}

fn main() {
    let args = Args::parse();
    match args.order {
        Order::Build => process_build(args),
        Order::Run => todo!("Not done"),
        Order::Init => init_project(),
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
    ExitCode::Ok.exit();
}
