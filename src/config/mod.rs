use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::collections::HashMap;
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)] pub projects: HashMap<String, ProjectConfig>,
    #[serde(default)] pub embedding: Option<EmbeddingConfig>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingConfig {
    pub api_base: String,
    #[serde(default)] pub api_key: Option<String>,
    pub model: String,
    #[serde(default)] pub dimension: Option<usize>,
    #[serde(default = "default_batch")] pub batch_size: usize,
    #[serde(default = "default_text_limit")] pub text_limit: usize,
    #[serde(default = "default_max_chars")] pub max_chars_per_batch: usize,
}
fn default_batch() -> usize { 64 }
fn default_text_limit() -> usize { 300 }
fn default_max_chars() -> usize { 90000 }
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub repos: HashMap<String, RepoConfig>,
    #[serde(default)] pub base_db: Option<String>,
    #[serde(default = "dft")] pub similarity_threshold: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RepoConfig {
    pub root: String,
    #[serde(default)] pub languages: Vec<String>,
}
fn dft() -> f64 { 0.75 }
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let p = Self::path()?;
        if !p.exists() { return Ok(Config::default()); }
        Ok(serde_yaml::from_str(&std::fs::read_to_string(&p)?)?)
    }
    pub fn data_dir() -> anyhow::Result<PathBuf> {
        let d = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("HOME"))?.join(".codeloom");
        std::fs::create_dir_all(&d)?; Ok(d)
    }
    fn path() -> anyhow::Result<PathBuf> { Ok(Self::data_dir()?.join("config.yaml")) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_config_default() { assert!(Config::default().projects.is_empty()); }
    #[test] fn test_data_dir_returns_path() { assert!(Config::data_dir().unwrap().to_string_lossy().contains(".codeloom")); }
    #[test] fn test_dft_threshold() { assert_eq!(dft(), 0.75); }
}
