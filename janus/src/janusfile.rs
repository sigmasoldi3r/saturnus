use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct JanusBuild {
    pub source: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
pub struct JanusProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JanusWorkspaceConfig {
    #[serde(rename = "type")]
    pub project_type: String,
    pub project: Option<JanusProject>,
    pub build: Option<JanusBuild>,
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
