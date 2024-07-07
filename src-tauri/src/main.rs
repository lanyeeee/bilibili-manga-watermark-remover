// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::unwrap_used)]
mod types;
mod utils;
mod watermark;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
fn generate_background(
    manga_dir: &str,
    rect_data: types::RectData,
    height: u32,
    width: u32,
) -> (String, String) {
    match watermark::generate_background(manga_dir, &rect_data, height, width) {
        Ok((black_path, white_path)) => (black_path, white_path),
        // FIXME: 处理异常
        Err(err) => {
            println!("{err:?}");
            (String::new(), String::new())
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
fn open_image(path: String) -> Option<types::JpgImageData> {
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
    Some(types::JpgImageData {
        info: types::JpgImageInfo {
            height,
            width,
            path,
        },
        src,
    })
}

#[tauri::command(async)]
#[specta::specta]
fn remove_watermark(manga_dir: &str, output_dir: &str) -> Option<String> {
    use std::fmt::Write;
    match watermark::remove(manga_dir, output_dir) {
        Ok(()) => None,
        // FIXME: 处理异常
        Err(err) => {
            let msg = err
                .chain()
                .enumerate()
                .fold(String::new(), |mut output, (i, e)| {
                    let _ = writeln!(output, "{i}: {e}");
                    output
                });
            Some(msg)
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
fn background_exists(is_black: bool) -> bool {
    let Ok(exe_dir_path) = utils::get_exe_dir_path() else {
        return false;
    };
    let filename = if is_black { "black.png" } else { "white.png" };
    exe_dir_path.join(filename).exists()
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
fn get_image_size_count(manga_dir: &str) -> Vec<types::ImageSizeCount> {
    // 用于存储不同尺寸的图片的数量
    let mut size_count = std::collections::HashMap::new();
    // 遍历漫画目录下的所有文件，统计不同尺寸的图片的数量
    for entry in walkdir::WalkDir::new(std::path::Path::new(manga_dir))
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.into_path();
        if path.is_file() && path.extension().map_or(false, |e| e == "jpg") {
            let Ok(size) = imagesize::size(&path) else {
                continue;
            };
            let key = (size.height as u32, size.width as u32);
            let count = size_count.entry(key).or_insert(0);
            *count += 1;
        }
    }
    // 将统计结果转换为Vec<ImageSizeCount>
    let mut image_size_count: Vec<types::ImageSizeCount> = size_count
        .into_iter()
        .map(|((height, width), count)| types::ImageSizeCount {
            height,
            width,
            count,
        })
        .collect();
    // 以count降序排序
    image_size_count.sort_by(|a, b| b.count.cmp(&a.count));
    // 返回结果
    image_size_count
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
fn get_jpg_image_infos(manga_dir: &str) -> Vec<types::JpgImageInfo> {
    // 用于存储jpg图片的信息
    let mut jpg_image_infos = vec![];
    // 遍历漫画目录下的所有文件，获取jpg图片的信息
    for entry in walkdir::WalkDir::new(std::path::Path::new(manga_dir))
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.into_path();
        if path.is_file() && path.extension().map_or(false, |e| e == "jpg") {
            let Ok(size) = imagesize::size(&path) else {
                continue;
            };
            jpg_image_infos.push(types::JpgImageInfo {
                height: size.height as u32,
                width: size.width as u32,
                path: path.display().to_string(),
            });
        }
    }
    jpg_image_infos
}

fn main() -> anyhow::Result<()> {
    let invoke_handler = {
        let builder = tauri_specta::ts::builder()
            .commands(tauri_specta::collect_commands![
                generate_background,
                remove_watermark,
                background_exists,
                open_image,
                get_image_size_count,
                get_jpg_image_infos,
            ])
            .header("// @ts-nocheck"); // 跳过检查，避免__makeEvents__错误

        #[cfg(debug_assertions)] // 只有在debug模式下才会生成bindings.ts
        let builder = builder.path("../src/bindings.ts");

        builder.build()?
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
