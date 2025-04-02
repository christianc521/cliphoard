use anyhow::{Ok, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Debug, Copy, Serialize, Clone)]
pub enum ColorConfig {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub background: ColorConfig,
    pub cursor: ColorConfig,
    pub font_size: u16,
    pub selected_color: ColorConfig,
    pub unselected_color: ColorConfig,
    pub input_color: ColorConfig,
    pub width: i32,
    pub height: i32,
}

impl Config {
    pub fn load() -> Result<Self> {
        let proj_dir = ProjectDirs::from("com", "cogStudios", "cliphoard")
            .expect("Could not locate project directory.");

        let config_dir = proj_dir.config_dir();
        let config_path = config_dir.join("config.toml");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        // Load or create config file
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&config)?)
        } else {
            let default_config = Config::default();
            let config = toml::to_string_pretty(&default_config)?;
            fs::write(config_path, config)?;
            Ok(default_config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            background: ColorConfig::RGBA(60, 56, 42, 80),
            cursor: ColorConfig::RGB(20, 200, 29),
            font_size: 24,
            selected_color: ColorConfig::RGB(230, 230, 230),
            unselected_color: ColorConfig::RGBA(230, 230, 230, 70),
            input_color: ColorConfig::RGB(255, 255, 255),
            width: 1000,
            height: 230,
        }
    }
}
