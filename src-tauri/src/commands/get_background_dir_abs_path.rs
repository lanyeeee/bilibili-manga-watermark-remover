use std::path::PathBuf;

use tauri::AppHandle;

use crate::errors::CommandResult;
use crate::types::CommandResponse;
use crate::utils;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_background_dir_abs_path(
    app: AppHandle,
    manga_dir: &str,
    width: u32,
    height: u32,
) -> CommandResult<CommandResponse<PathBuf>> {
    let abs_path = utils::get_background_dir_abs_path(&app, manga_dir, width, height)?;
    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: abs_path,
    })
}
