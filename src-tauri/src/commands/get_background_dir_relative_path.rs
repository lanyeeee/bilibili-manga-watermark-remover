use std::path::PathBuf;

use crate::errors::CommandResult;
use crate::utils;

#[tauri::command(async)]
#[specta::specta]
pub fn get_background_dir_relative_path(
    manga_dir: &str,
    width: u32,
    height: u32,
) -> CommandResult<PathBuf> {
    let relative_path = utils::get_background_dir_relative_path(manga_dir, width, height)?;
    Ok(relative_path)
}
