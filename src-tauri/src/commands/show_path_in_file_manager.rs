use std::path::PathBuf;

#[tauri::command(async)]
#[specta::specta]
pub fn show_path_in_file_manager(path: &str) {
    let path = PathBuf::from(path);
    showfile::show_path_in_file_manager(path);
}
