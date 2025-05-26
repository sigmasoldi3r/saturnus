use std::{
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use args::Args;
use clap::Parser;
use colored::Colorize;
use config::{Dependency, DependencyContainer, Project};
use lockcdb::{LockDb, Package};
use regex::Regex;
use walkdir::WalkDir;

mod args;
mod cmd;
mod config;
mod lockcdb;
mod logging;

const DEFAULT_PATH: &'static str = "titan.toml";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn default_path() -> PathBuf {
    PathBuf::from_str(DEFAULT_PATH).unwrap()
}

fn load_conf(path: impl AsRef<Path>) -> Result<Project> {
    let raw = std::fs::read_to_string(path).map_err(Box::new)?;
    let proj: Project = toml::from_str(raw.as_str()).map_err(Box::new)?;
    Ok(proj)
}

#[derive(Debug, Clone)]
struct SimpleErr(String);
impl std::fmt::Display for SimpleErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}
impl Error for SimpleErr {}
impl SimpleErr {
    fn wrap(err: impl std::error::Error) -> Self {
        Self(format!("{err}"))
    }
}

const TARGET_CACHE: &'static str = "target/deps";

fn download_dep(db: &mut LockDb, record: &Package) -> std::result::Result<(), SimpleErr> {
    println!("Fetching {}...", record.url);
    let cwd = PathBuf::from_str(TARGET_CACHE).map_err(SimpleErr::wrap)?;
    let id = db.store(record);
    cmd::git_clone(record.url.clone(), cwd, id.to_string())
        .map_err(|err| SimpleErr(format!("{err}")))?;
    Ok(())
}

fn prepare_dep(
    (_key, entry): (&String, &DependencyContainer),
) -> std::result::Result<(), SimpleErr> {
    let dep = entry.clone();
    let dep = dep.unwrap();
    let Dependency::Git { git: url, version } = dep else {
        return Err(SimpleErr("Only git dependency scheme supported".into()));
    };
    let pkg = Package { url, version };
    let mut db = LockDb::load().map_err(SimpleErr::wrap)?;
    if db.contains(&pkg) {
        // Skip download! No more need.
    } else {
        download_dep(&mut db, &pkg)?;
        db.save().map_err(SimpleErr::wrap)?;
    }
    Ok(())
}

fn collect_files(
    path: impl AsRef<Path>,
) -> std::result::Result<Vec<walkdir::DirEntry>, impl Error> {
    WalkDir::new(path)
        .into_iter()
        .try_fold(Vec::new(), |mut acc, item| match item {
            Ok(ok) => {
                acc.push(ok);
                Ok(acc)
            }
            Err(err) => Err(err),
        })
        .map(|v| {
            v.into_iter()
                .filter(|e| e.path().is_file())
                .collect::<Vec<_>>()
        })
        .map_err(Box::new)
}

fn batch_compile(project: &Project) -> Result<()> {
    let replace_root = Regex::new(r#"[/\\]?src[\\/]"#).unwrap();
    let replace_slash = Regex::new(r#"[/\\]"#).unwrap();
    let replace_ext = Regex::new(r#".st$"#).unwrap();
    cmd::mock_target_folders().map_err(Box::new)?;
    warn!("Dependency resolving is being worked on.");
    for dep in project.dependencies.iter() {
        prepare_dep(dep)?;
    }
    LockDb::load().map_err(Box::new)?.save().map_err(Box::new)?;
    info!("Compiling objects...");
    let sources = collect_files("src")?;
    progress_bar::init_progress_bar(sources.len());
    for entry in sources {
        let path = entry.clone().into_path();
        let out_path = {
            let mut path = path.clone();
            path.set_extension("");
            path
        };
        let path_str = out_path.into_os_string().into_string().unwrap();
        let ext = if let Some(ext) = path.extension() {
            format!(".{}", ext.to_str().unwrap())
        } else {
            String::new()
        };
        let path_str = format!("{path_str}{ext}");
        println!("  {} {path_str}...", format!("Compiling").green());
        let object_name = replace_root.replace(&path_str, "");
        let object_name = replace_slash.replace_all(&object_name, "_");
        let object_name = replace_ext.replace(&object_name, "");
        let object_name = format!("target/objects/{object_name}.lua");
        let mod_path = replace_root.replace(&path_str, "");
        let mod_path = replace_ext.replace(&mod_path, "");
        let mod_path = mod_path.to_string();
        cmd::saturnc(path_str, mod_path, object_name)?;
        progress_bar::print_progress_bar_info(
            "Compiled",
            entry.path().to_str().unwrap(),
            progress_bar::Color::Green,
            progress_bar::Style::Bold,
        );
        progress_bar::inc_progress_bar();
    }
    progress_bar::finalize_progress_bar();
    let objects = collect_files("target/objects")?;
    info!("Linking objects...");
    progress_bar::init_progress_bar(objects.len());
    let mut output = File::create("target/_collect_.lua").map_err(Box::new)?;
    if !project.linking.no_std {
        let out = cmd::produce_std_code()?;
        write!(output, "do{out}\nend\n").map_err(Box::new)?;
    }
    for entry in objects {
        let content = std::fs::read_to_string(entry.path()).map_err(Box::new)?;
        write!(output, "do{}\nend\n", content).map_err(Box::new)?;
        progress_bar::print_progress_bar_info(
            "Linked",
            entry.path().to_str().unwrap(),
            progress_bar::Color::Green,
            progress_bar::Style::Bold,
        );
        progress_bar::inc_progress_bar();
    }
    progress_bar::finalize_progress_bar();
    info!("Building {}", project.package.name);
    match project.linking.mode {
        config::LinkMode::Collect => {
            let fout = format!("target/{}.lua", project.package.name);
            std::fs::copy("target/_collect_.lua", fout).map_err(Box::new)?;
        }
        config::LinkMode::Binary => todo!("Binary compilation not ready yet."),
        config::LinkMode::PreserveStructure => todo!("Preserve structure not ready yet."),
    }
    std::fs::remove_file("target/_collect_.lua").map_err(Box::new)?;
    info!("Done");
    Ok(())
}

fn run(project: Option<PathBuf>) -> Result<()> {
    let conf_path = project.unwrap_or(default_path());
    let conf = load_conf(conf_path)?;
    batch_compile(&conf)?;
    Ok(())
}

fn compile(project: Option<PathBuf>) -> Result<()> {
    let conf_path = project.unwrap_or(default_path());
    let conf = load_conf(conf_path)?;
    batch_compile(&conf)?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    let result = match args {
        Args::Build { project } => run(project),
        Args::Run { project } => compile(project),
        Args::New {} => todo!(),
        Args::Init {} => todo!(),
        Args::Add {} => todo!(),
        Args::Test {} => todo!(),
    };

    match result {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", format!("{err}").red())
        }
    }
}
