use std::{path::PathBuf, process::Command};

use console::style;

use crate::{
    deps::resolve_deps,
    dir::create_dist_dirs,
    errors::ExitCode,
    janusfile::{DependencyList, JanusBuild},
};

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

#[derive(PartialEq)]
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

    /// Attempt to compile a single file.
    fn compile_file(&self, info: &CompilationInfo, file_path: &PathBuf) {
        let mut cmd = Command::new("saturnus");
        if info.mode == CompilationMode::Bin && &info.main == file_path {
            println!("Pronic MAIN MAUIII");
        }
        let mut out = info
            .output
            .join("cache")
            .join(file_path.strip_prefix(&info.source).unwrap());
        match info.target {
            CompilationTarget::Lua => out.set_extension("lua"),
        };
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        cmd.arg("-c")
            .arg(file_path.to_str().unwrap())
            .arg("-o")
            .arg(out);
        // Handle the final execution of the command.
        match cmd.spawn() {
            Ok(mut proc) => match proc.wait() {
                Ok(exit) => {
                    if !exit.success() {
                        eprintln!(
                            "Could not compile {:?}, compiler exited with code {}",
                            file_path,
                            exit.code().map_or("unknown".to_owned(), |i| i.to_string())
                        );
                        ExitCode::FailedCompilation.exit();
                    }
                }
                Err(err) => {
                    eprintln!("Could not compile {:?}, {}", file_path, err);
                    ExitCode::FailedCompilation.exit();
                }
            },
            Err(err) => {
                eprintln!("Could not compile {:?}, {}", file_path, err);
                ExitCode::FailedCompilation.exit();
            }
        }
    }

    /// Compilation entry point
    pub fn compile(
        self,
        mode: CompilationMode,
        dependencies: DependencyList,
        info: JanusBuild,
    ) -> Result {
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
        let main = main.unwrap_or("src/main.saturn".into());
        let no_std = no_std.unwrap_or(false);
        let target = match target.as_str() {
            "Lua" => CompilationTarget::Lua,
            _ => {
                eprintln!("Target {} not supported", target);
                ExitCode::TargetNotSupported.exit();
            }
        };
        let module_system = match module_system.as_str() {
            "sam" => ModuleSystem::Sam,
            "native" => ModuleSystem::Native,
            _ => {
                eprintln!("Unknown module system '{}'!", module_system);
                ExitCode::UnknownModuleSystem.exit();
            }
        };
        let info = CompilationInfo {
            mode,
            output,
            source,
            compact,
            target,
            module_system,
            main,
            no_std,
        };
        // Those two steps cause exit if failed
        create_dist_dirs(&info.output);
        resolve_deps(&info, dependencies);
        let paths = glob::glob("./**/*.saturn").unwrap();
        let total = glob::glob("./**/*.saturn").unwrap().count();
        let mut i = 0;
        for entry in paths {
            match entry {
                Ok(entry) => {
                    i += 1;
                    println!(
                        "- {} {}{}",
                        style(format!("{i}/{total}")).color256(9_u8).italic(),
                        style(format!("Compiling {:?}", entry))
                            .color256(8_u8)
                            .italic(),
                        style("...").color256(8_u8).italic()
                    );
                    self.compile_file(&info, &entry);
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
