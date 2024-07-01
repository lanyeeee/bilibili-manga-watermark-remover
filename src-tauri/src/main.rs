// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use image::{ImageBuffer, RgbImage};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

mod types;

#[tauri::command(async)]
fn generate_background(image_path: &str, rect_data: types::RectData, is_black: bool) {
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
    // 找出出现次数最多的RGB值，即矩形框内的主要颜色，作为背景颜色
    let background_rgb = *color_count
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap()
        .0;
    // 将不在矩形框内的其他像素点设为背景颜色
    for y in 0..height {
        for x in 0..width {
            if x < left || x > right || y < top || y > bottom {
                img.put_pixel(x, y, background_rgb);
            }
        }
    }
    // 保存生成的背景图片
    if is_black {
        img.save("black.png").unwrap();
    } else {
        img.save("white.png").unwrap();
    }
}

#[tauri::command(async)]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|err| err.to_string())
}

//TODO: 代码需要重构
#[tauri::command(async)]
fn remove_watermark(manga_dir: &str, output_dir: &str) {
    let white = image::open("white.png").unwrap().to_rgb8();
    let black = image::open("black.png").unwrap().to_rgb8();
    let manga_dir = std::path::Path::new(manga_dir);
    let output_dir = std::path::Path::new(output_dir);
    // 用于将像素点的值限制在指定范围内
    let clamp = |value: f32, min: f32, max: f32| -> f32 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    };
    // 用于存储每个目录下所有图片的路径
    let mut directory_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    // 这里忽略了遍历过程中遇到的任何错误
    for entry in WalkDir::new(Path::new(manga_dir))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry: DirEntry = entry;
        let path: PathBuf = entry.path().to_path_buf();
        // 如果不是文件或不是jpg则跳过
        if !path.is_file() || path.extension().unwrap() != "jpg" {
            continue;
        }
        if let Some(parent) = path.parent() {
            // 将文件路径添加到对应目录的vector中
            directory_map
                .entry(parent.to_path_buf())
                .or_default()
                .push(path);
        }
    }
    // 遍历directory_map，对每个目录下的所有图片进行去除水印操作
    for (dir, files) in directory_map.iter() {
        // 使用rayon的并行迭代器，并行处理每个图片
        files
            .into_par_iter()
            .try_for_each(|in_image_path| -> Result<(), image::ImageError> {
                let relative_path = in_image_path
                    .strip_prefix(manga_dir.parent().unwrap())
                    .unwrap();
                let out_image_path = output_dir.join(relative_path);
                let in_img = image::open(&in_image_path)
                    .expect("Failed to open output image")
                    .to_rgb8();
                if in_img.width() != white.width() || in_img.height() != white.height() {
                    return Ok(());
                }
                // 创建一个新的图片缓冲区，用于存储去除水印后的图片
                let mut img_buf = ImageBuffer::new(in_img.width(), in_img.height());
                // 这里如果用rayon的并行反而会导致性能下降，所以使用普通的for循环
                for y in 0..in_img.height() {
                    for x in 0..in_img.width() {
                        let [in_r, in_g, in_b] = in_img.get_pixel(x, y).0;
                        let [black_r, black_g, black_b] = black.get_pixel(x, y).0;
                        let [white_r, white_g, white_b] = white.get_pixel(x, y).0;

                        let watermark_removed_pixel = [
                            (in_r as f32 - black_r as f32) / ((white_r - black_r) as f32 / 255.0),
                            (in_g as f32 - black_g as f32) / ((white_g - black_g) as f32 / 255.0),
                            (in_b as f32 - black_b as f32) / ((white_b - black_b) as f32 / 255.0),
                        ];

                        let clamped_pixel = image::Rgb([
                            clamp(watermark_removed_pixel[0], 0.0, 255.0) as u8,
                            clamp(watermark_removed_pixel[1], 0.0, 255.0) as u8,
                            clamp(watermark_removed_pixel[2], 0.0, 255.0) as u8,
                        ]);

                        img_buf.put_pixel(x, y, clamped_pixel);
                    }
                }
                // 保证输出目录存在
                fs::create_dir_all(out_image_path.parent().unwrap()).unwrap();
                // 保存去除水印后的图片
                let watermark_removed_image = RgbImage::from(img_buf);
                let encoder = jpeg_encoder::Encoder::new_file(out_image_path, 97).unwrap();
                encoder
                    .encode(
                        &watermark_removed_image.as_raw(),
                        watermark_removed_image.width() as u16,
                        watermark_removed_image.height() as u16,
                        jpeg_encoder::ColorType::Rgb,
                    )
                    .unwrap();
                Ok(())
            })
            .unwrap();
    }
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
