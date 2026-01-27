use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FilmrConfig {
    pub custom_stocks_path: PathBuf,
}

pub struct ConfigManager {
    pub config: FilmrConfig,
    pub root_path: PathBuf,
    pub config_path: PathBuf,
}

impl ConfigManager {
    pub fn init() -> Option<Self> {
        let user_dirs = UserDirs::new()?;
        let home = user_dirs.home_dir();
        let root_path = home.join(".filmr");
        let config_path = root_path.join("config.json");
        let default_stocks_path = root_path.join("stocks");

        if !root_path.exists() {
            let _ = fs::create_dir_all(&root_path);
        }
        if !default_stocks_path.exists() {
            let _ = fs::create_dir_all(&default_stocks_path);
        }

        let config = if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                serde_json::from_str(&content).unwrap_or_else(|_| FilmrConfig {
                    custom_stocks_path: default_stocks_path.clone(),
                })
            } else {
                FilmrConfig {
                    custom_stocks_path: default_stocks_path.clone(),
                }
            }
        } else {
            let config = FilmrConfig {
                custom_stocks_path: default_stocks_path.clone(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&config) {
                let _ = fs::write(&config_path, json);
            }
            config
        };

        Some(Self {
            config,
            root_path,
            config_path,
        })
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write(&self.config_path, json);
        }
    }
}
