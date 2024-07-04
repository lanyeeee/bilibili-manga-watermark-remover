#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RectData {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JpgImage {
    pub width: u32,
    pub height: u32,
    pub src: String,
    pub path: String,
}
