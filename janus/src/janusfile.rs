use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ModulesOptions {
    pub external: Option<Vec<PathBuf>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PathBufOrPathBufList {
    PathBuf(PathBuf),
    PathBufList(Vec<PathBuf>),
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JanusBuild {
    pub source: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub main: Option<PathBufOrPathBufList>,
    pub format: Option<String>,
    pub target: Option<String>,
    pub module_system: Option<String>,
    pub no_std: Option<bool>,
    pub modules: Option<ModulesOptions>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct JanusProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DependencyObject {
    pub version: Option<String>,
    pub git: Option<String>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OutputFormat {
    File,
    Directory,
    FlatDirectory,
    Binary,
    Zip,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DependencyDef {
    PlainVersion(String),
    Options(DependencyObject),
}

pub type DependencyList = HashMap<String, DependencyDef>;

#[derive(Debug, Deserialize, Serialize)]
pub struct JanusWorkspaceConfig {
    #[serde(rename = "type")]
    pub project_type: String,
    pub project: Option<JanusProject>,
    pub build: Option<JanusBuild>,
    pub dependencies: Option<DependencyList>,
}

/// Resolves the janus file
fn get_janus_file(root: &Option<PathBuf>) -> std::io::Result<String> {
    if let Some(root) = root {
        fs::read_to_string(root.join("Janus.toml"))
            .or_else(|_| fs::read_to_string(root.join("janus.toml")))
    } else {
        fs::read_to_string("Janus.toml").or_else(|_| fs::read_to_string("janus.toml"))
    }
}

fn handle_cannot_read_file(err: &std::io::Error) -> Option<JanusWorkspaceConfig> {
    eprintln!(
        "Could not read the Janus.toml project file! {}",
        err.to_string()
    );
    None
}

fn handle_parsing_error(err: &toml::de::Error) -> Option<JanusWorkspaceConfig> {
    eprintln!(
        "There's something wrong with the Janus project file: {}",
        err.to_string()
    );
    None
}

impl JanusWorkspaceConfig {
    pub fn parse_janus_file(root: &Option<PathBuf>) -> Option<JanusWorkspaceConfig> {
        let result = get_janus_file(root);
        match result {
            Ok(content) => {
                let result = toml::from_str::<JanusWorkspaceConfig>(content.as_str());
                match result {
                    Ok(config) => Some(config),
                    Err(err) => handle_parsing_error(&err),
                }
            }
            Err(err) => handle_cannot_read_file(&err),
        }
    }
}
