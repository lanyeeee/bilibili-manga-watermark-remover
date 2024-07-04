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
fn open_image(path: String) -> Option<types::JpgImage> {
    let img = image::open(&path).ok()?.to_rgb8();
    // 获取图片的宽高
    let (width, height) = img.dimensions();
    // 创建一个内存缓冲区，用于存储图片数据
    let mut image_data: Vec<u8> = vec![];
    let mut cursor = std::io::Cursor::new(&mut image_data);
    // 将图片数据写入内存缓冲区
    img.write_to(&mut cursor, image::ImageFormat::Jpeg).ok()?;
    // 将图片数据转换为base64编码
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);
    // 构建图片的src属性
    let src = format!("data:image/jpeg;base64,{base64}");
    // 返回JpgImage对象
    Some(types::JpgImage {
        width: width,
        height: height,
        src: src,
        path: path,
    })
}

#[tauri::command(async)]
fn open_background(is_black: bool) -> Option<types::JpgImage> {
    let path = if is_black { "black.png" } else { "white.png" };
    open_image(path.to_string())
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
            open_image,
            open_background,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
