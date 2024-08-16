use std::path::PathBuf;

use crate::errors::CommandResult;
use crate::types::CommandResponse;
use crate::utils;

#[tauri::command(async)]
#[specta::specta]
pub fn get_background_dir_relative_path(
    manga_dir: &str,
    width: u32,
    height: u32,
) -> CommandResult<CommandResponse<PathBuf>> {
    let relative_path = utils::get_background_dir_relative_path(manga_dir, width, height)?;
    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: relative_path,
    })
}
