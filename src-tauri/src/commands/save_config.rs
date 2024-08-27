use std::sync::RwLock;

use tauri::{AppHandle, State};

use crate::config::Config;
use crate::errors::CommandResult;
use crate::extensions::IgnoreRwLockPoison;
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn save_config(
    app: AppHandle,
    config_state: State<RwLock<Config>>,
    config: Config,
) -> CommandResult<CommandResponse<()>> {
    let mut config_state = config_state.write_or_panic();
    *config_state = config;
    config_state.save(&app)?;
    Ok(CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    })
}
