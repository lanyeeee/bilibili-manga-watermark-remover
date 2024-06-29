// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn generate_black_or_white(
    image_path: &str,
    rect_data: types::RectData,
    is_black: bool,
) -> Result<(), ()> {
    let mut img = image::open(image_path).unwrap().to_rgb8();
    // 获取图片的宽高
    let (width, height) = img.dimensions();
    // 获取矩形框的左上角和右下角坐标
    let (left, top) = (rect_data.left, rect_data.top);
    let (right, bottom) = (rect_data.right, rect_data.bottom);
    // 用于统计矩形框内每种颜色的像素点数量
    let mut color_count = std::collections::HashMap::new();
    // 统计矩形框内每种颜色的像素点数量
    for y in top..=bottom {
        for x in left..=right {
            let pixel = img.get_pixel(x, y);
            let count = color_count.entry(pixel).or_insert(0);
            *count += 1;
        }
    }
    // 找出出现次数最多的RGB值，即矩形框内的主要颜色
    let dominant_rgb = *color_count
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap()
        .0;
    // 将不在矩形框内的其他像素点设为主要颜色
    for y in 0..height {
        for x in 0..width {
            if x < left || x > right || y < top || y > bottom {
                img.put_pixel(x, y, dominant_rgb);
            }
        }
    }
    // 保存图片
    if is_black {
        img.save("black.png").unwrap();
    } else {
        img.save("white.png").unwrap();
    }
    Ok(())
}

#[tauri::command]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_black_or_white,
            read_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
