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
pub struct ImageSizeCount {
    pub height: u32,
    pub width: u32,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct JpgImageInfo {
    pub height: u32,
    pub width: u32,
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct JpgImageData {
    pub info: JpgImageInfo,
    pub src: String,
}
