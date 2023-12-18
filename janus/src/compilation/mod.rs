pub mod pipelines;
pub mod utils;

use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::Command,
};

use console::style;
use copy_dir::copy_dir;

use crate::{
    compilation::utils::{get_output_folder, get_source_folder},
    deps::resolve_deps,
    dir::create_dist_dirs,
    display::get_bar,
    errors::ExitCode,
    janusfile::{DependencyList, JanusBuild, JanusProject, OutputFormat},
};

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
    pub format: OutputFormat,
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
    fn compile_file(&self, info: &CompilationInfo, file_path: &PathBuf) -> PathBuf {
        let mut cmd = Command::new("saturnus");
        if info.mode == CompilationMode::Bin && &info.main == file_path {
            // Here we should inject STD if no no-std flag is provided.
        }
        let mut out = info
            .output
            .join("cache")
            .join("objects")
            .join(file_path.strip_prefix(&info.source).unwrap());
        match info.target {
            CompilationTarget::Lua => out.set_extension("lua"),
        };
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        cmd.arg("-c")
            .arg(file_path.to_str().unwrap())
            .arg("-o")
            .arg(&out);
        if &info.main != file_path {
            // If main, omit no STD
            cmd.arg("--no-std");
        }
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
        out
    }

    /// Post compilation collection step
    fn collect_objects(
        &self,
        project: &JanusProject,
        info: &CompilationInfo,
        objects: Vec<PathBuf>,
        external_modules: &HashSet<PathBuf>,
    ) {
        let objects_base_path = info.output.join("cache").join("objects");
        let target_base_path = info.output.join("target");
        println!("Linking artifacts...");
        match info.format {
            OutputFormat::File => {
                pipelines::FilePipeline.collect_file(
                    &info,
                    &objects,
                    &objects_base_path,
                    &target_base_path,
                    None,
                    external_modules,
                );
                if external_modules.len() > 0 {
                    println!("\nLinking additional artifacts...");
                    let pb = get_bar(objects.len() as u64);
                    for entry in external_modules.iter() {
                        let base_target = entry.strip_prefix(&objects_base_path).unwrap();
                        let target = target_base_path.join(base_target);
                        pb.set_message(format!("Linking additional source {:?}...", &target));
                        fs::create_dir_all(target.parent().unwrap()).unwrap();
                        fs::copy(entry, target).unwrap();
                        pb.inc(1);
                    }
                    pb.finish_with_message("Done");
                }
            }
            OutputFormat::Directory => {
                let pb = get_bar(objects.len() as u64 + 1);
                for entry in glob::glob("./dist/cache/objects/**/*.lua").unwrap() {
                    let entry = entry.expect("Failed to resolve glob dep on directory");
                    let base_target = entry.strip_prefix(&objects_base_path).unwrap();
                    let target = target_base_path.join(base_target);
                    pb.set_message(format!("Linking {:?}...", &target));
                    fs::create_dir_all(target.parent().unwrap()).unwrap();
                    fs::copy(entry, target).unwrap();
                    pb.inc(1);
                }
                pb.finish_with_message("Done");
            }
            OutputFormat::FlatDirectory => todo!("Directory flattening"),
            OutputFormat::Binary => {
                let path = info.output.join("cache").join("main.lua");
                pipelines::FilePipeline.collect_file(
                    &info,
                    &objects,
                    &objects_base_path,
                    &target_base_path,
                    Some(path.clone()),
                    external_modules,
                );
                #[cfg(target_family = "windows")]
                let binaries = include_bytes!("../../../target/release/runtime.exe");
                #[cfg(target_family = "unix")]
                let binaries = include_bytes!("../../../target/release/runtime");
                let out_path = project.name.clone().unwrap_or("main".into());
                let out_path = info.output.join("target").join(out_path);
                #[cfg(target_family = "windows")]
                let out_path = if let Some(ext) = out_path.extension() {
                    let ext: String = ext.to_string_lossy().into();
                    out_path.with_extension(ext + ".ext")
                } else {
                    out_path.with_extension("exe")
                };
                let mut open_options = OpenOptions::new();
                let out = open_options.write(true).create(true).truncate(true);
                #[cfg(target_family = "unix")]
                use std::os::unix::fs::OpenOptionsExt;
                #[cfg(target_family = "unix")]
                let mut out = out.mode(0o711);
                let mut out = out.open(out_path).unwrap();
                out.write_all(binaries).unwrap();
                let main_src = fs::read(&path).unwrap();
                out.write_all(&main_src).unwrap();
                out.write(&main_src.len().to_le_bytes()).unwrap();
            }
            OutputFormat::Zip => todo!("Zip file production"),
        }
    }

    /// Compilation entry point
    pub fn compile(
        self,
        mode: CompilationMode,
        dependencies: DependencyList,
        info: JanusBuild,
        meta: JanusProject,
    ) -> Result {
        let JanusBuild {
            output,
            source,
            main,
            target,
            module_system,
            no_std,
            format,
            modules,
        } = info;
        let output = get_output_folder(output);
        let source = get_source_folder(source);
        let main = main.unwrap_or("src/main.saturn".into());
        let no_std = no_std.unwrap_or(false);
        let format = match format.unwrap_or("dir".to_owned()).as_str() {
            "flat" => OutputFormat::FlatDirectory,
            "dir" | "plain" | "native" => OutputFormat::Directory,
            "file" | "compact" => OutputFormat::File,
            "binary" | "bin" | "exe" => OutputFormat::Binary,
            "zip" => OutputFormat::Zip,
            format => {
                eprintln!("Output format {format} not supported");
                ExitCode::UnknownModuleSystem.exit();
            }
        };
        let target = match target.unwrap_or("Lua".into()).as_str() {
            "Lua" => CompilationTarget::Lua,
            target => {
                eprintln!("Target {target} not supported");
                ExitCode::TargetNotSupported.exit();
            }
        };
        let module_system = match module_system.unwrap_or("sam".into()).as_str() {
            "sam" => ModuleSystem::Sam,
            "native" => ModuleSystem::Native,
            module_system => {
                eprintln!("Unknown module system '{module_system}'!");
                ExitCode::UnknownModuleSystem.exit();
            }
        };
        let info = CompilationInfo {
            mode,
            output,
            source,
            format,
            target,
            module_system,
            main,
            no_std,
        };
        // Those two steps cause exit if failed
        create_dist_dirs(&info.output);
        resolve_deps(&info, dependencies);
        let paths = glob::glob("./src/**/*.saturn").unwrap();
        let total = glob::glob("./src/**/*.saturn").unwrap().count();
        println!("Compiling sources...");
        let pb = get_bar(total as u64);
        let mut objects = Vec::<PathBuf>::new();
        let mut external_modules = HashSet::<PathBuf>::new();
        let external_modules_origin = {
            let mut set = HashSet::new();
            if let Some(mods) = modules {
                if let Some(ext) = mods.external {
                    for mod_path in ext {
                        set.insert(mod_path);
                    }
                }
            }
            set
        };
        for entry in paths {
            match entry {
                Ok(entry) => {
                    pb.set_message(format!("Compiling {:?}...", entry));
                    let object = self.compile_file(&info, &entry);
                    objects.push(object.clone());
                    if external_modules_origin.contains(&entry) {
                        external_modules.insert(object);
                    }
                    pb.inc(1);
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
        pb.finish_with_message("Done");
        self.collect_objects(&meta, &info, objects, &external_modules);
        Ok(())
    }
}
