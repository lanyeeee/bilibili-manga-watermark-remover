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
pub struct JpgImage {
    pub width: u32,
    pub height: u32,
    pub src: String,
    pub path: String,
}
