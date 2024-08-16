use tauri::AppHandle;

use crate::config::Config;
use crate::errors::CommandResult;
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(app: AppHandle) -> CommandResult<CommandResponse<Config>> {
    let config = Config::load(&app).map_err(anyhow::Error::from)?;
    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: config,
    })
}
