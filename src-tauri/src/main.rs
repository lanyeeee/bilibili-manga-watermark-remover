// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
mod watermark;

#[tauri::command(async)]
fn generate_background(image_path: &str, rect_data: types::RectData, is_black: bool) {
    watermark::generate_background(image_path, &rect_data, is_black);
}

#[tauri::command(async)]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|err| err.to_string())
}

#[tauri::command(async)]
fn remove_watermark(manga_dir: &str, output_dir: &str) {
    watermark::remove_manga_watermark(manga_dir, output_dir);
}

#[tauri::command(async)]
fn background_exists(is_black: bool) -> bool {
    let path = if is_black { "black.png" } else { "white.png" };
    std::path::Path::new(path).exists()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            generate_background,
            read_file,
            remove_watermark,
            background_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
