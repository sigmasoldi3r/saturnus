use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use console::style;

use crate::{
    deps::resolve_deps,
    dir::create_dist_dirs,
    display::get_bar,
    errors::ExitCode,
    janusfile::{DependencyList, JanusBuild, JanusProject, OutputFormat},
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
    fn collect_objects(&self, info: &CompilationInfo, objects: Vec<PathBuf>) {
        let base = info.output.join("cache").join("objects");
        let target = info.output.join("target");
        println!("Linking artifacts...");
        match info.format {
            OutputFormat::File => {
                let pb = get_bar(objects.len() as u64);
                let mut main: Option<PathBuf> = None;

                let mut file_out = match info.target {
                    CompilationTarget::Lua => {
                        let mut file_out =
                            File::create(info.output.join("target").join("main.lua")).unwrap();
                        file_out
                            .write_all(
                                b"local __modules__ = {};
do
  local __native_require__ = require;
  require = function(fp)
    if __modules__[fp] ~= nil then
      if package.loaded[fp] == nil then
        package.loaded[fp] = __modules__[fp]();
      end
      return package.loaded[fp];
    end
    return __native_require__(fp);
  end;
end",
                            )
                            .unwrap();
                        file_out
                    }
                };
                let mut main_path = base.join(info.main.strip_prefix(&info.source).unwrap());
                main_path.set_extension("lua");
                for entry in objects.iter() {
                    if entry == &main_path {
                        main = Some(entry.clone());
                        continue;
                    }
                    let base_target = entry.strip_prefix(&base).unwrap();
                    let target = target.join(base_target);
                    pb.set_message(format!("Linking {:?}...", &target));
                    let src = fs::read_to_string(entry).unwrap();
                    let mut path_name = entry.clone();
                    path_name.set_extension("");
                    let mut path_name = path_name.strip_prefix(&base).unwrap();
                    if path_name.file_name().unwrap() == "init" {
                        path_name = path_name.parent().unwrap();
                    }
                    let path_name = path_name
                        .to_string_lossy()
                        .to_string()
                        .replace("\\", ".")
                        .replace("/", ".");
                    file_out
                        .write_fmt(format_args!(
                            "\n__modules__[\"{}\"] = function()\n",
                            path_name
                        ))
                        .unwrap();
                    file_out.write_all(&src.as_bytes()).unwrap();
                    file_out.write(b"\nend;").unwrap();
                    pb.inc(1);
                }
                if let Some(entry) = main {
                    pb.set_message("Linking standard library...");
                    file_out
                        .write(b"\n__modules__[\"std\"] = function()\n")
                        .unwrap();
                    file_out
                        .write_all(fs::read_to_string(base.join("std.lua")).unwrap().as_bytes())
                        .unwrap();
                    file_out.write(b"\nend;").unwrap();
                    pb.set_message("Collecting main file...");
                    let src = fs::read_to_string(entry).unwrap();
                    file_out.write(b"\n").unwrap();
                    file_out.write_all(&src.as_bytes()).unwrap();
                    pb.inc(1);
                }
                pb.finish_with_message("Done");
            }
            OutputFormat::Directory => {
                let pb = get_bar(objects.len() as u64);
                for entry in objects.iter() {
                    let base_target = entry.strip_prefix(&base).unwrap();
                    let target = target.join(base_target);
                    pb.set_message(format!("Linking {:?}...", &target));
                    fs::create_dir_all(target.parent().unwrap()).unwrap();
                    fs::copy(entry, target).unwrap();
                    pb.inc(1);
                }
                pb.finish_with_message("Done");
            }
            OutputFormat::FlatDirectory => todo!("Directory flattening"),
            OutputFormat::Binary => todo!("Binary file production"),
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
        } = info;
        let output = get_output_folder(output);
        let source = get_source_folder(source);
        let main = main.unwrap_or("src/main.saturn".into());
        let no_std = no_std.unwrap_or(false);
        let format = match format.unwrap_or("dir".to_owned()).as_str() {
            "flat" => OutputFormat::FlatDirectory,
            "dir" => OutputFormat::Directory,
            "file" => OutputFormat::File,
            "binary" => OutputFormat::Binary,
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
        let paths = glob::glob("./**/*.saturn").unwrap();
        let total = glob::glob("./**/*.saturn").unwrap().count();
        println!("Compiling sources...");
        let pb = get_bar(total as u64);
        let mut objects = Vec::<PathBuf>::new();
        for entry in paths {
            match entry {
                Ok(entry) => {
                    pb.set_message(format!("Compiling {:?}...", entry));
                    objects.push(self.compile_file(&info, &entry));
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
        self.collect_objects(&info, objects);
        Ok(())
    }
}
