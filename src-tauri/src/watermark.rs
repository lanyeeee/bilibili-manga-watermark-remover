use std::{
    collections::HashMap,
    env::current_exe,
    path::{Path, PathBuf},
    sync::Mutex,
};

use image::{Rgb, RgbImage};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::types;

pub fn generate_background(
    manga_dir: &str,
    rect_data: &types::RectData,
    height: u32,
    width: u32,
) -> (String, String) {
    // 遍历manga_dir目录下的所有jpg文件，收集尺寸符合要求的图片的路径
    let image_paths: Vec<PathBuf> = WalkDir::new(Path::new(manga_dir))
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.into_path();
            if !path.is_file() || path.extension()? != "jpg" {
                return None;
            }
            let size = imagesize::size(&path).ok()?;
            if size.width as u32 == width && size.height as u32 == height {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let black_path: Mutex<Option<String>> = Mutex::new(None);
    let white_path: Mutex<Option<String>> = Mutex::new(None);
    // 并发遍历image_paths
    image_paths.par_iter().for_each(|path| {
        // 如果black_path和white_path都已经找到了，则直接跳过
        if black_path.lock().unwrap().is_some() && white_path.lock().unwrap().is_some() {
            return;
        }
        let mut img = image::open(path).unwrap().to_rgb8();
        let (left, top) = (rect_data.left, rect_data.top);
        let (right, bottom) = (rect_data.right, rect_data.bottom);
        // 检查图片是否满足黑色或白色背景的条件
        let is_black = match is_black_background(&img, rect_data) {
            Some(is_black) => is_black,
            None => return, // 如果既不是黑色背景也不是白色背景，则跳过
        };
        // 获取左上角的颜色
        let color = *img.get_pixel(left, top);
        // 把截图区域外的像素点设置为左上角的颜色
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x < left || x > right || y < top || y > bottom {
                *pixel = color;
            }
        }
        let exe_path = current_exe().unwrap();
        let exe_dir_path = exe_path.parent().unwrap();
        let filename = if is_black { "black.png" } else { "white.png" };
        let output_path = exe_dir_path.join(filename);
        // 保存黑色背景或白色背景的图片
        let mut background_path = if is_black {
            black_path.lock().unwrap()
        } else {
            white_path.lock().unwrap()
        };
        // 如果background_path是None，则把output_path赋值给background_path，并保存图片
        if background_path.is_none() {
            *background_path = Some(output_path.to_str().unwrap().to_string());
            // 因为save是耗时操作，所以在这里手动释放锁
            drop(background_path);
            img.save(&output_path).unwrap();
        }
    });
    // 获取黑色背景和白色背景的图片路径
    let black_path = black_path.lock().unwrap().as_ref().unwrap().clone();
    let white_path = white_path.lock().unwrap().as_ref().unwrap().clone();
    // 返回黑色背景和白色背景的图片路径
    (black_path, white_path)
}

/// 检查图片是否满足黑色或白色背景的条件，如果为None则表示既不满足黑色背景的条件也不满足白色背景的条件
fn is_black_background(img: &RgbImage, rect_data: &types::RectData) -> Option<bool> {
    let (left, top) = (rect_data.left, rect_data.top);
    let (right, bottom) = (rect_data.right, rect_data.bottom);
    // 获取左上角的颜色
    let color = *img.get_pixel(left, top);
    let [r, g, b] = color.0;
    // 如果r,g,b通道之间不相等，则不满足黑色背景或白色背景的条件
    if r != g || g != b {
        return None;
    }
    // 如果截图区域的左右两边的颜色有一个与左上角的颜色不同，则不满足黑色背景或白色背景的条件
    for y in top..=bottom {
        if img.get_pixel(left, y) != &color || img.get_pixel(right, y) != &color {
            return None;
        }
    }
    // 如果截图区域的上下两边的颜色有一个与左上角的颜色不同，则不满足黑色背景或白色背景的条件
    for x in left..=right {
        if img.get_pixel(x, top) != &color || img.get_pixel(x, bottom) != &color {
            return None;
        }
    }
    // 如果所有通道的值都小于25，则认为是黑色背景
    let is_black = r <= 25;
    // 如果所有通道的值都大于230，并且截图区域内的通道值都大于100(小于100一般是页码)，则认为是白色背景
    let is_white = r >= 230
        && img
            .enumerate_pixels()
            .filter(|(x, y, _)| x >= &left && x <= &right && y >= &top && y <= &bottom) //矩形区域内的像素
            .all(|(_, _, pixel)| pixel.0[0] > 100); // 通道值大于100
    match (is_black, is_white) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
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
            save_jpg_image(&img, &out_image_path);
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

/// 保存jpg图片到指定路径
fn save_jpg_image(img: &RgbImage, path: &Path) {
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
