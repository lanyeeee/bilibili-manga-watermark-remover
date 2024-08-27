use std::sync::RwLock;

use tauri::State;

use crate::config::Config;
use crate::extensions::IgnoreRwLockPoison;
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config: State<'_, RwLock<Config>>) -> CommandResponse<Config> {
    CommandResponse {
        code: 0,
        msg: String::new(),
        data: config.read_or_panic().clone(),
    }
}
