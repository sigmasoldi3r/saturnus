use std::path::PathBuf;

use console::style;

use crate::{errors::ExitCode, janusfile::JanusBuild};

fn get_output_folder(output: Option<PathBuf>) -> PathBuf {
    if let Some(output) = output {
        output
    } else {
        println!(
            "{}",
            style("Dist folder not specified, will default to dist")
                .color256(8_u8)
                .italic()
                .dim()
        );
        "dist".into()
    }
}

fn get_source_folder(source: Option<PathBuf>) -> PathBuf {
    if let Some(source) = source {
        source
    } else {
        println!(
            "{}",
            style("Source folder not specified, will default to src")
                .color256(8_u8)
                .italic()
                .dim()
        );
        "src".into()
    }
}

pub enum CompilationError {}

pub type Result = std::result::Result<(), CompilationError>;

pub enum CompilationMode {
    Lib,
    Bin,
}

pub enum CompilationTarget {
    Lua,
}
pub enum ModuleSystem {
    Native,
    Sam,
}

pub struct CompilationInfo {
    pub output: PathBuf,
    pub source: PathBuf,
    pub compact: bool,
    pub target: CompilationTarget,
    pub module_system: ModuleSystem,
    pub main: PathBuf,
    pub no_std: bool,
    pub mode: CompilationMode,
}

pub struct CompilationHost {}
impl CompilationHost {
    pub fn new() -> CompilationHost {
        CompilationHost {}
    }
    fn compile_file(info: &CompilationInfo) {}
    pub fn compile(self, mode: CompilationMode, info: JanusBuild) -> Result {
        let JanusBuild {
            output,
            source,
            main,
            compact,
            target,
            module_system,
            no_std,
        } = info;
        let output = get_output_folder(output);
        let source = get_source_folder(source);
        let compact = compact.unwrap_or(true);
        let target = target.unwrap_or("Lua".into());
        let module_system = module_system.unwrap_or("sam".into());
        let main = main.unwrap_or("main".into());
        let no_std = no_std.unwrap_or(false);
        for entry in glob::glob("./**/*.saturn").unwrap() {
            match entry {
                Ok(entry) => {
                    print!("{}", style(format!("Compiling {:?}...", entry)).dim());
                }
                Err(err) => {
                    eprintln!(
                        "{}",
                        style(format!(
                            "Error reading {:?}! -> {}",
                            err.path(),
                            err.error()
                        ))
                        .red()
                        .reverse()
                        .bold()
                    );
                    ExitCode::CannotOpenFile.exit();
                }
            }
        }
        Ok(())
    }
}
