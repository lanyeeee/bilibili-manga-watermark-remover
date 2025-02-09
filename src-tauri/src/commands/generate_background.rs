use std::path::PathBuf;

use anyhow::{anyhow, Context};
use image::RgbImage;
use parking_lot::Mutex;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tauri::AppHandle;
use walkdir::WalkDir;

use crate::errors::CommandResult;
use crate::types::RectData;
use crate::utils;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
pub fn generate_background(
    app: AppHandle,
    manga_dir: &str,
    rect_data: Option<RectData>,
    width: u32,
    height: u32,
) -> CommandResult<()> {
    let output_dir = utils::get_background_dir_abs_path(&app, manga_dir, width, height)?;
    // TODO: 给RectData实现Default trait，以替换下面的代码
    let default_rect_data = RectData {
        left: (width as f32 * 0.835) as u32,
        top: (height as f32 * 0.946) as u32,
        right: (width as f32 * 0.994) as u32,
        bottom: (height as f32 * 0.994) as u32,
    };
    let rect_data = rect_data.unwrap_or(default_rect_data);

    // 保证输出目录存在
    std::fs::create_dir_all(&output_dir).context(format!("创建目录 {output_dir:?} 失败"))?;
    // 收集尺寸符合width和height的图片的路径
    let image_paths = create_image_paths(manga_dir, width, height);
    // 用于保存各种符合条件的背景水印图
    let backgrounds = Mutex::new(vec![]);
    // 用于标记是否找到了黑色和白色背景水印图
    let background_pair_found = Mutex::new(false);
    // 并发遍历image_paths
    let image_paths = image_paths.par_iter();
    image_paths.try_for_each(|path| -> anyhow::Result<()> {
        // 如果已经找到了黑色和白色背景水印图，则直接返回
        if *background_pair_found.lock() {
            return Ok(());
        }

        let mut img = image::open(path)
            .context(format!("打开图片 {path:?} 失败"))?
            .to_rgb8();
        // 如果图片不满足背景的条件，则直接跳过
        if !is_background(&img, &rect_data) {
            return Ok(());
        };
        // 获取左上角的颜色
        let (left, top) = (rect_data.left, rect_data.top);
        let (right, bottom) = (rect_data.right, rect_data.bottom);
        let color = *img.get_pixel(left, top);
        // 把截图区域外的像素点设置为左上角的颜色
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x < left || x > right || y < top || y > bottom {
                *pixel = color;
            }
        }
        let mut backgrounds = backgrounds.lock();
        backgrounds.push(img);
        // 按照像素值排序，保证黑色背景水印图在前，白色背景水印图在后
        backgrounds.sort_by(|a, b| {
            let a_color = a.get_pixel(0, 0);
            let b_color = b.get_pixel(0, 0);
            a_color[0].cmp(&b_color[0])
        });
        if backgrounds.len() < 2 {
            return Ok(());
        }

        let black = &backgrounds[0];
        let white = &backgrounds[backgrounds.len() - 1];
        // 如果黑色和白色背景水印图的像素值差异大于50，则认为找到了黑色和白色背景水印图
        let black_color = black.get_pixel(0, 0);
        let white_color = white.get_pixel(0, 0);
        if white_color[0] - black_color[0] > 50 {
            *background_pair_found.lock() = true;
        }

        Ok(())
    })?;

    let backgrounds = std::mem::take(&mut *backgrounds.lock());
    let background_pair_found = std::mem::take(&mut *background_pair_found.lock());
    // 如果有第一张背景水印图，则将其保存为黑色背景
    if let Some(black) = backgrounds.first() {
        let black_output_path = output_dir.join("black.png");
        black
            .save(&black_output_path)
            .context(format!("保存图片 {black_output_path:?} 失败",))?;
    }
    // 如果找到了黑色和白色背景水印图
    if background_pair_found {
        // 把最后一张背景水印图保存为白色背景
        let white = &backgrounds[backgrounds.len() - 1];
        let white_output_path = output_dir.join("white.png");
        white
            .save(&white_output_path)
            .context(format!("保存图片 {white_output_path:?} 失败",))?;
    }

    if backgrounds.is_empty() {
        return Err(anyhow!("找不到尺寸为({width}x{height})的背景水印图\n").into());
    } else if !background_pair_found {
        return Err(anyhow!("只找到一张尺寸为({width}x{height})的背景水印图\n").into());
    };

    Ok(())
}

/// 遍历`manga_dir`目录下的所有jpg文件，收集尺寸符合`width`和`height`的图片的路径
#[allow(clippy::cast_possible_truncation)]
fn create_image_paths(manga_dir: &str, width: u32, height: u32) -> Vec<PathBuf> {
    let image_paths: Vec<PathBuf> = WalkDir::new(PathBuf::from(manga_dir))
        .max_depth(2) // 一般第一层目录是章节目录，第二层目录是图片文件
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.into_path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?.to_lowercase();
            if ext != "jpg" && ext != "jpeg" {
                return None;
            }
            // 只收集尺寸符合width和height的图片的路径
            let (img_width, img_height) = image::image_dimensions(&path).ok()?;
            if img_width == width && img_height == height {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    image_paths
}

/// 检查图片`img`是否满足背景的条件
#[allow(clippy::cast_precision_loss)]
fn is_background(img: &RgbImage, rect_data: &RectData) -> bool {
    let (left, top) = (rect_data.left, rect_data.top);
    let (right, bottom) = (rect_data.right, rect_data.bottom);
    let inside_rect = |x: u32, y: u32| x >= left && x <= right && y >= top && y <= bottom;
    // 获取左上角的颜色
    let color = *img.get_pixel(left, top);
    let [r, g, b] = color.0;
    // 如果r,g,b通道之间不相等，则不满足背景的条件
    if r != g || g != b {
        return false;
    }
    // 如果截图区域的左右两边的颜色有一个与左上角的颜色不同，则不满足背景的条件
    for y in top..=bottom {
        if img.get_pixel(left, y) != &color || img.get_pixel(right, y) != &color {
            return false;
        }
    }
    // 如果截图区域的上下两边的颜色有一个与左上角的颜色不同，则不满足背景的条件
    for x in left..=right {
        if img.get_pixel(x, top) != &color || img.get_pixel(x, bottom) != &color {
            return false;
        }
    }
    // 统计rect_data区域内color颜色的像素点数量
    let color_count = img
        .enumerate_pixels()
        .filter(|(x, y, &pixel)| inside_rect(*x, *y) && pixel == color)
        .count();
    // 如果rect_data区域内的像素点数量大于总数的90%，则不满足背景的条件
    if color_count as f32 / ((right - left + 1) * (bottom - top + 1)) as f32 > 0.9 {
        return false;
    }
    true
}
