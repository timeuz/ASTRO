use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub notify_on_completion: bool,
    pub muted_categories: Vec<String>,
    pub neutral_mode: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            notify_on_completion: true,
            muted_categories: Vec::new(),
            neutral_mode: false,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    fn get_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/astro-agent-companion/user_config.json")
    }

    pub fn load() -> UserConfig {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&contents) {
                    return config;
                }
            }
        }
        UserConfig::default()
    }

    pub fn save(config: &UserConfig) {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(contents) = serde_json::to_string_pretty(config) {
            if let Err(e) = fs::write(&path, contents) {
                error!("Failed to save user config: {}", e);
            }
        }
    }
}
