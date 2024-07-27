use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkStartEventPayload {
    pub dir_path: PathBuf,
    pub total: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkStartEvent(pub RemoveWatermarkStartEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkSuccessEventPayload {
    pub dir_path: PathBuf,
    pub img_path: PathBuf,
    pub current: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkSuccessEvent(pub RemoveWatermarkSuccessEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkErrorEventPayload {
    pub dir_path: PathBuf,
    pub img_path: PathBuf,
    pub err_msg: String,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkErrorEvent(pub RemoveWatermarkErrorEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct RemoveWatermarkEndEventPayload {
    pub dir_path: PathBuf,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct RemoveWatermarkEndEvent(pub RemoveWatermarkEndEventPayload);
