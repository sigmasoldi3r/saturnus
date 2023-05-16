mod jobs;

use std::fs;

use clap::{Parser, Subcommand};
use console::style;
use glob::glob;
use indicatif::ProgressBar;
use serde::Deserialize;
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    order: Order,
}

#[derive(Subcommand)]
enum Order {
    Build {},
}

#[derive(Debug, Deserialize, Default)]
struct JanusBuild {
    sources: Option<Vec<String>>,
    output: Option<String>,
    additional_sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct JanusProject {
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JanusWorkspaceConfig {
    project: Option<JanusProject>,
    build: Option<JanusBuild>,
}

fn parse_janus_file() -> Option<JanusWorkspaceConfig> {
    let result = fs::read_to_string("Janus.toml");
    match result {
        Ok(content) => {
            let result = toml::from_str::<JanusWorkspaceConfig>(content.as_str());
            match result {
                Ok(config) => Some(config),
                Err(err) => {
                    println!(
                        "There's something wrong with the Janus project file: {}",
                        err.to_string()
                    );
                    None
                }
            }
        }
        Err(err) => {
            println!(
                "Could not read the Janus.toml project file! {}",
                err.to_string()
            );
            None
        }
    }
}

fn collect_sources() -> Option<Vec<String>> {
    let result = glob("./**/*.saturn")
        .expect("GLOB ERROR")
        .map(|x| x.unwrap().to_str().unwrap().to_string().trim().to_string())
        .collect();
    Some(result)
}

fn process_build() -> Option<()> {
    let JanusWorkspaceConfig { project, build } = parse_janus_file()?;
    let project = project.unwrap_or_default();
    let build = build.unwrap_or_default();
    println!("{} Collecting project info...", style("[1/4]").bold());
    if project.name.is_none() {
        println!(
            "{} Project file does not have a name!",
            style("WARNING:").yellow().bold()
        );
    }
    if project.version.is_none() {
        println!(
            "{} Project file does not have a version!",
            style("WARNING:").yellow().bold()
        );
    }
    let tab = "     ";
    if build.output.is_none() {
        println!(
            "{} {}",
            tab,
            style("All files will be compiled in-place")
                .color256(8_u8)
                .italic()
                .dim()
        );
    }
    println!("{} Collecting sources...", style("[2/4]").bold());
    let sources = collect_sources()?;
    // In the future this process will be parallelized
    println!("{} Compiling sources...", style("[3/4]").bold());
    let pb = ProgressBar::new(sources.len() as u64);
    for source in sources {
        let result = Command::new("saturnus")
            .args(vec!["-i", source.as_str()])
            .output();
        if result.is_err() {
            eprintln!(
                "\n{} Failed to compile {}!",
                style("FATAL:").bold().red(),
                style(source.clone()).bold()
            );
        }
        let output = result.unwrap();
        if !output.status.success() {
            eprintln!(
                "\nFailed to compile {}!\n{}\n{} {}",
                style(source.clone()).bold().underlined(),
                style(String::from_utf8(output.stdout).unwrap())
                    .dim()
                    .color256(8_u8),
                style("FATAL:").bold().red(),
                String::from_utf8(output.stderr).unwrap()
            );
            return None;
        }
        pb.inc(1);
    }
    println!("{} Done!", style("[4/4]").bold());
    Some(())
}

fn main() {
    let args = Args::parse();
    match args.order {
        Order::Build {} => process_build().unwrap(),
    }
}
