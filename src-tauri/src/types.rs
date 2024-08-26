use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct CommandResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

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
    pub black_background: Option<JpgImageData>,
    #[serde(rename = "whiteBackground")]
    pub white_background: Option<JpgImageData>,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct JpgImageInfo {
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct JpgImageData {
    pub info: JpgImageInfo,
    pub base64: String,
}
impl JpgImageData {
    pub fn to_image(&self) -> anyhow::Result<image::DynamicImage> {
        let decode = general_purpose::STANDARD.decode(self.base64.as_bytes())?;
        let image = image::load_from_memory(&decode)?;
        Ok(image)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub enum QrCodeStatus {
    NotScan,
    Scanning,
    Complete(String),
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct QrCodeData {
    pub base64: String,
    #[serde(rename = "qrcodeKey")]
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub ep_id: i64,
    pub ep_title: String,
    pub comic_id: i64,
    pub comic_title: String,
    pub is_locked: bool,
    pub is_downloaded: bool,
}
