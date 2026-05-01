use serde::{Serialize,Deserialize};
use std::path::PathBuf;
use std::collections::HashMap;
#[derive(Debug,Serialize,Deserialize,Default)]
pub struct Config { pub projects: HashMap<String, ProjectConfig> }
#[derive(Debug,Serialize,Deserialize)]
pub struct ProjectConfig { pub repos: HashMap<String, RepoConfig>, #[serde(default)] pub base_db: Option<String>, #[serde(default="dft")] pub similarity_threshold: f64 }
#[derive(Debug,Serialize,Deserialize)]
pub struct RepoConfig { pub root: String, #[serde(default)] pub languages: Vec<String> }
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