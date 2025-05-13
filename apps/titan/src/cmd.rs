use std::{error::Error, path::PathBuf, process::Command};

use colored::Colorize;

pub fn mock_target_folders() -> std::io::Result<()> {
    std::fs::create_dir_all("target/deps")?;
    std::fs::create_dir_all("target/objects")?;
    Ok(())
}

pub fn git_clone(target: String, cwd: PathBuf, folder_name: String) -> Result<(), Box<dyn Error>> {
    Command::new("git")
        .current_dir(cwd)
        .args(vec!["clone".into(), target, folder_name])
        .output()
        .map_err(Box::new)?;
    Ok(())
}

pub fn saturnc(input: String, mod_path: String, output: String) -> Result<(), Box<dyn Error>> {
    let out = Command::new("saturnc")
        .args(vec![
            "compile".into(),
            format!("--mod-path={mod_path}"),
            format!("-i={input}"),
            format!("-o={output}"),
        ])
        .output()
        .map_err(Box::new)?;
    if !out.status.success() {
        println!(
            "{}",
            format!("{}", String::from_utf8(out.stderr).unwrap()).red()
        );
    }
    Ok(())
}

pub fn produce_std_code() -> Result<String, Box<dyn Error>> {
    let out = Command::new("saturnc")
        .args(vec!["std-output", "--stdout"])
        .output()
        .map_err(Box::new)?;
    if !out.status.success() {
        println!(
            "{}",
            format!("{}", String::from_utf8(out.stderr).unwrap()).red()
        );
    }
    Ok(String::from_utf8(out.stdout).unwrap())
}
