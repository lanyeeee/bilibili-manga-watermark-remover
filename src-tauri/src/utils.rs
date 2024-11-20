use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

pub fn get_background_dir_relative_path(
    manga_dir: &str,
    width: u32,
    height: u32,
) -> anyhow::Result<PathBuf> {
    let manga_dir_name = Path::new(manga_dir)
        .file_name()
        .ok_or(anyhow::anyhow!("获取漫画目录名失败"))?
        .to_str()
        .ok_or(anyhow::anyhow!("漫画目录名包含非UTF-8字符"))?;
    let relative_path = format!("背景水印图/{manga_dir_name}{width}x{height}");
    Ok(PathBuf::from(relative_path))
}

pub fn get_background_dir_abs_path(
    app: &AppHandle,
    manga_dir: &str,
    width: u32,
    height: u32,
) -> anyhow::Result<PathBuf> {
    let resource_dir = app.path().resource_dir()?;
    let relative_path = get_background_dir_relative_path(manga_dir, width, height)?;
    let abs_path = resource_dir.join(relative_path);
    Ok(abs_path)
}
