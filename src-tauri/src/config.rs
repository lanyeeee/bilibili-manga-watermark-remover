use crate::types::ImageFormat;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Config {
    #[serde(rename = "outputDir")]
    pub output_dir: PathBuf,
    #[serde(rename = "outputFormat")]
    pub output_format: ImageFormat,
    #[serde(rename = "outputOptimize")]
    pub output_optimize: bool,
}

impl Config {
    pub fn load(app: &AppHandle) -> anyhow::Result<Self> {
        let resource_dir = app.path().resource_dir()?;
        let config_path = resource_dir.join("config.json");
        let default_config = Config {
            output_dir: resource_dir,
            output_format: ImageFormat::Jpeg,
            output_optimize: false,
        };
        // 如果配置文件存在且能够解析，则使用配置文件中的配置，否则使用默认配置
        let config = if config_path.exists() {
            let config_string = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&config_string).unwrap_or(default_config)
        } else {
            default_config
        };
        config.save(app)?;

        Ok(config)
    }
    // FIXME: config_state需要被修改
    pub fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let resource_dir = app.path().resource_dir()?;
        let config_path = resource_dir.join("config.json");
        let config_string = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_string)?;
        Ok(())
    }
}
