use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageIndexData {
    pub host: String,
    pub images: Vec<Image>,
    #[serde(rename = "last_modified")]
    pub last_modified: String,
    pub path: String,
    pub video: Video,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub path: String,
    #[serde(rename = "video_path")]
    pub video_path: String,
    #[serde(rename = "video_size")]
    pub video_size: String,
    pub x: i64,
    pub y: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "bin_url")]
    pub bin_url: String,
    pub filename: String,
    #[serde(rename = "img_urls")]
    pub img_urls: Vec<Value>,
    #[serde(rename = "img_x_len")]
    pub img_x_len: i64,
    #[serde(rename = "img_x_size")]
    pub img_x_size: i64,
    #[serde(rename = "img_y_len")]
    pub img_y_len: i64,
    #[serde(rename = "img_y_size")]
    pub img_y_size: i64,
    #[serde(rename = "raw_height")]
    pub raw_height: String,
    #[serde(rename = "raw_rotate")]
    pub raw_rotate: String,
    #[serde(rename = "raw_width")]
    pub raw_width: String,
    pub resource: Vec<Value>,
    pub route: String,
    pub svid: String,
}
