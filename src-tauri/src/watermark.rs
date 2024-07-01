use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use image::{Rgb, RgbImage};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::types;

/// 生成带水印的背景图片
pub fn generate_background(image_path: &str, rect_data: &types::RectData, is_black: bool) {
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

/// 移除manga_dir目录下所有图片的水印，并保存到output_dir目录
pub fn remove_manga_watermark(manga_dir: &str, output_dir: &str) {
    let manga_dir = Path::new(manga_dir);
    let output_dir = Path::new(output_dir);
    let white = image::open("white.png").unwrap().to_rgb8();
    let black = image::open("black.png").unwrap().to_rgb8();
    // 构建一个HashMap，key是目录的路径，value是该目录下的所有jpg文件的路径
    let directory_map = build_directory_map(manga_dir);
    // 遍历directory_map，对每个目录下的所有图片进行去除水印操作
    for (_dir, files) in directory_map.iter() {
        // 使用rayon的并行迭代器，并行处理每个目录下的所有图片
        files.into_par_iter().for_each(|img_path| {
            // 获取相对路径
            let relative_path = img_path.strip_prefix(manga_dir.parent().unwrap()).unwrap();
            // 构建输出图片的路径
            let out_image_path = output_dir.join(relative_path);
            // 打开输入图片
            let mut img = image::open(img_path).unwrap().to_rgb8();
            // 去除水印
            remove_image_watermark(&white, &black, &mut img);
            // 保存去除水印后的图片(无论是否成功去除水印都会保存)
            save_image(&img, &out_image_path);
        });
    }
}

/// 限制value的值在min和max之间
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// 构建一个HashMap，key是目录的路径，value是该目录下的所有jpg文件的路径
fn build_directory_map(manga_dir: &Path) -> HashMap<PathBuf, Vec<PathBuf>> {
    let mut directory_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    // 遍历manga_dir目录下的所有文件和子目录
    for entry in WalkDir::new(manga_dir).into_iter().filter_map(Result::ok) {
        let entry: DirEntry = entry;
        let path = entry.into_path();
        // 如果是文件且是jpg文件则添加到directory_map中
        if path.is_file() && path.extension().map_or(false, |e| e == "jpg") {
            if let Some(parent) = path.parent() {
                directory_map
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push(path);
            }
        }
    }
    directory_map
}

/// 去除img的水印
fn remove_image_watermark(white: &RgbImage, black: &RgbImage, img: &mut RgbImage) {
    // TODO: 处理图片大小不一致的情况
    if img.width() != white.width() || img.height() != white.height() {
        return;
    }
    // 遍历图片的每个像素点
    for (x, y, img_pixel) in img.enumerate_pixels_mut() {
        let [img_r, img_g, img_b] = img_pixel.0;
        let [black_r, black_g, black_b] = black.get_pixel(x, y).0;
        let [white_r, white_g, white_b] = white.get_pixel(x, y).0;
        // 计算去除水印后的像素点值
        let watermark_removed_pixel = [
            (img_r as f32 - black_r as f32) / ((white_r - black_r) as f32 / 255.0),
            (img_g as f32 - black_g as f32) / ((white_g - black_g) as f32 / 255.0),
            (img_b as f32 - black_b as f32) / ((white_b - black_b) as f32 / 255.0),
        ];
        // 将像素点的值限制在0-255之间
        let clamped_pixel = Rgb([
            clamp(watermark_removed_pixel[0], 0.0, 255.0) as u8,
            clamp(watermark_removed_pixel[1], 0.0, 255.0) as u8,
            clamp(watermark_removed_pixel[2], 0.0, 255.0) as u8,
        ]);
        // 将去除水印后的像素点值写入到图片缓冲区中
        *img_pixel = clamped_pixel;
    }
}

/// 保存图片到指定路径
fn save_image(img: &RgbImage, path: &Path) {
    // 保证输出目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    // 保存去除水印后的图片，使用jpeg_encoder库的Encoder，效率更高
    let encoder = jpeg_encoder::Encoder::new_file(path, 97).unwrap();
    encoder
        .encode(
            &img.as_raw(),
            img.width() as u16,
            img.height() as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .unwrap();
}
