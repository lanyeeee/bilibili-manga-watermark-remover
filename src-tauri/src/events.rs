use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tauri_specta::Event;

//FIXME: 统一风格
#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkStartEventPayload {
    pub dir_path: PathBuf,
    pub total: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkStartEvent(pub RemoveWatermarkStartEventPayload);
//FIXME: 统一风格
#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkSuccessEventPayload {
    pub dir_path: PathBuf,
    pub img_path: PathBuf,
    pub current: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkSuccessEvent(pub RemoveWatermarkSuccessEventPayload);
//FIXME: 统一风格
#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkErrorEventPayload {
    pub dir_path: PathBuf,
    pub img_path: PathBuf,
    pub err_msg: String,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkErrorEvent(pub RemoveWatermarkErrorEventPayload);
//FIXME: 统一风格
#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkEndEventPayload {
    pub dir_path: PathBuf,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkEndEvent(pub RemoveWatermarkEndEventPayload);
