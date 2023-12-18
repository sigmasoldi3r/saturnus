use std::{
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use copy_dir::copy_dir;
use indicatif::ProgressBar;

use crate::{
    compilation::CompilationInfo,
    display::get_bar,
    janusfile::{DependencyList, DependencyObject},
};

fn resolve_dep_options(name: &String, options: &DependencyObject, bar: &ProgressBar) {
    let old_msg = bar.message();
    if !Path::new(&format!("dist/dependencies/{name}")).exists() {
        if let Some(git) = options.git.clone() {
            bar.set_message(format!("Cloning {git}..."));
            let status = Command::new("git")
                .arg("clone")
                .arg(git)
                .arg(name)
                .current_dir("dist/dependencies")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("Failed to resolve the dependency!");
            if !status.success() {
                panic!("Failed to resolve the dependency!");
            }
        }
    }
    bar.set_message(format!("Compiling {name}..."));
    let output = Command::new("janus")
        .arg("build")
        .current_dir(format!("dist/dependencies/{name}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to compile the dependency!");
    if !output.status.success() {
        panic!(
            "Failed to compile '{}' dependency!\nReason:\n{}",
            name,
            String::from_utf8_lossy(&output.stderr),
        );
    }
    bar.set_message(format!("Copying {name} artifacts..."));
    if !Path::new(&format!("dist/cache/objects/{name}")).exists() {
        copy_dir(
            format!("dist/dependencies/{name}/dist/cache/objects"),
            format!("dist/cache/objects/{name}"),
        )
        .expect("Failed to copy dependency artifacts!");
    }
    bar.set_message(old_msg);
}

pub fn resolve_deps(info: &CompilationInfo, dependencies: DependencyList) {
    let _ = info; // TODO!
    let pb = get_bar(dependencies.len() as u64);
    println!("Resolving dependencies...");
    for (name, dep) in dependencies.into_iter() {
        pb.set_message(format!("Linking {}...", name));
        match dep {
            crate::janusfile::DependencyDef::PlainVersion(_) => todo!(),
            crate::janusfile::DependencyDef::Options(options) => {
                resolve_dep_options(&name, &options, &pb)
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done");
}
