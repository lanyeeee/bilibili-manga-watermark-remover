use tauri::AppHandle;

use crate::config::Config;
use crate::errors::CommandResult;
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn save_config(app: AppHandle, config: Config) -> CommandResult<CommandResponse<()>> {
    config.save(&app)?;
    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    })
}
