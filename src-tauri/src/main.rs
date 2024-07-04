// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
mod watermark;

#[tauri::command(async)]
#[specta::specta]
fn generate_background(image_path: &str, rect_data: types::RectData, is_black: bool) {
    watermark::generate_background(image_path, &rect_data, is_black);
}

#[tauri::command(async)]
#[specta::specta]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|err| err.to_string())
}

#[tauri::command(async)]
#[specta::specta]
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
#[specta::specta]
fn open_background(is_black: bool) -> Option<types::JpgImage> {
    let path = if is_black { "black.png" } else { "white.png" };
    open_image(path.to_string())
}

#[tauri::command(async)]
#[specta::specta]
fn remove_watermark(manga_dir: &str, output_dir: &str) {
    watermark::remove_manga_watermark(manga_dir, output_dir);
}

#[tauri::command(async)]
#[specta::specta]
fn background_exists(is_black: bool) -> bool {
    let path = if is_black { "black.png" } else { "white.png" };
    std::path::Path::new(path).exists()
}

#[tauri::command(async)]
#[specta::specta]
fn get_manga_sizes(manga_dir: &str) -> Vec<types::MangaSize> {
    // 用于存储不同尺寸的图片的数量
    let mut size_count = std::collections::HashMap::new();
    // 遍历漫画目录下的所有文件，统计不同尺寸的图片的数量
    for entry in walkdir::WalkDir::new(std::path::Path::new(manga_dir))
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.into_path();
        if path.is_file() && path.extension().map_or(false, |e| e == "jpg") {
            let size = imagesize::size(&path).unwrap();
            let (height, width) = (size.height as u32, size.width as u32);
            let key = (height, width);
            let count = size_count.entry(key).or_insert(0);
            *count += 1;
        }
    }
    // 将统计结果转换为MangaSize对象的Vec，并以count降序排序
    let mut manga_sizes: Vec<types::MangaSize> = size_count
        .into_iter()
        .map(|((height, width), count)| types::MangaSize {
            width: width,
            height: height,
            count: count,
        })
        .collect();
    manga_sizes.sort_by(|a, b| b.count.cmp(&a.count));
    // 返回MangaSize对象的Vec
    manga_sizes
}

fn main() {
    let invoke_handler = {
        let builder = tauri_specta::ts::builder()
            .commands(tauri_specta::collect_commands![
                generate_background,
                read_file,
                remove_watermark,
                background_exists,
                open_image,
                open_background,
                get_manga_sizes,
            ])
            .header("// @ts-nocheck\n"); // <- This this appended to the start of the file;

        #[cfg(debug_assertions)] // <- Only export on non-release builds
        let builder = builder.path("../src/bindings.ts");

        builder.build().unwrap()
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
