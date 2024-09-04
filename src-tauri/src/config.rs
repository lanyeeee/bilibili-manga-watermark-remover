use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::types::ImageFormat;

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub output_dir: PathBuf,
    pub output_format: ImageFormat,
    pub output_optimize: bool,
    pub bili_cookie: String,
}

impl Config {
    pub fn new(app: &AppHandle) -> anyhow::Result<Self> {
        let resource_dir = app.path().resource_dir()?;
        let config_path = resource_dir.join("config.json");
        let default_config = Config {
            output_dir: resource_dir,
            output_format: ImageFormat::Jpeg,
            output_optimize: false,
            bili_cookie: String::new(),
        };
        let config = if config_path.exists() {
            let config_string = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&config_string).unwrap_or(default_config)
        } else {
            default_config
        };
        config.save(&app)?;
        Ok(config)
    }

    pub fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let resource_dir = app.path().resource_dir()?;
        let config_path = resource_dir.join("config.json");
        let config_string = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_string)?;
        Ok(())
    }

    pub fn get_cookie(&self) -> String {
        format!("SESSDATA={}", self.bili_cookie)
    }
}
