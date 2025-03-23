use std::path::PathBuf;

use anyhow::Context;

use crate::errors::CommandResult;
use crate::types::{ImageData, ImageInfo};

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
pub fn open_image(path: String) -> CommandResult<ImageData> {
    let path = PathBuf::from(path);
    let (width, height) = image::image_dimensions(&path)
        .context(format!("获取图片 {path:?} 的尺寸失败"))
        .map_err(anyhow::Error::from)?;
    let data = std::fs::read(&path)
        .context(format!("读取图片 {path:?} 失败"))
        .map_err(anyhow::Error::from)?;

    let data = ImageData {
        info: ImageInfo {
            width,
            height,
            path,
        },
        data,
    };

    Ok(data)
}
