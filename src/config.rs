use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Context;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub path: String,
    pub delta: f64,
    #[serde(default)]
    pub dry: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub groups: Vec<Group>,
    pub root_path: Option<String>,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_commit_message")]
    pub commit_message: String,
}

fn default_branch() -> String {
    "dev".to_string()
}

fn default_commit_message() -> String {
    "Force deploy".to_string()
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        // Try parsing as JSON first, then TOML if needed (though serde_json is usually specific)
        // For now, let's assume JSON based on the prompt "e.g. json, toml". 
        // I'll implement JSON first.
        let config: Config = serde_json::from_str(&content)
            .with_context(|| "Failed to parse config file as JSON")?;
            
        Ok(config)
    }
}
