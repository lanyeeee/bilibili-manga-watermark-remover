use parking_lot::RwLock;
use tauri::State;

use crate::config::Config;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config: State<'_, RwLock<Config>>) -> Config {
    config.read().clone()
}
