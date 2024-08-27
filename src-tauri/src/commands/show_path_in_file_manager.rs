use std::path::PathBuf;

use path_slash::PathBufExt;

#[tauri::command(async)]
#[specta::specta]
pub fn show_path_in_file_manager(path: &str) {
    let path = PathBuf::from_slash(path);
    showfile::show_path_in_file_manager(path);
}
