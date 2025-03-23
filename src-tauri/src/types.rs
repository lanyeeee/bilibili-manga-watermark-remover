use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct RectData {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct MangaDirData {
    pub width: u32,
    pub height: u32,
    pub count: u32,
    #[serde(rename = "blackBackground")]
    pub black_background: Option<ImageData>,
    #[serde(rename = "whiteBackground")]
    pub white_background: Option<ImageData>,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct ImageData {
    pub info: ImageInfo,
    pub data: Vec<u8>,
}
impl ImageData {
    pub fn to_image(&self) -> anyhow::Result<image::DynamicImage> {
        let image = image::load_from_memory(&self.data)?;
        Ok(image)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub enum ImageFormat {
    Jpeg,
    Png,
}
