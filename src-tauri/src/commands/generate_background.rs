use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use image::RgbImage;
use path_slash::PathBufExt;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tauri::AppHandle;
use walkdir::WalkDir;

use crate::errors::CommandResult;
use crate::extensions::IgnoreLockPoison;
use crate::types::{CommandResponse, RectData};
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
) -> CommandResult<CommandResponse<()>> {
    let output_dir = utils::get_background_dir_abs_path(&app, manga_dir, width, height)?;
    let default_rect_data = RectData {
        left: (width as f32 * 0.835) as u32,
        top: (height as f32 * 0.946) as u32,
        right: (width as f32 * 0.994) as u32,
        bottom: (height as f32 * 0.994) as u32,
    };
    let rect_data = rect_data.unwrap_or(default_rect_data);
    // let res = watermark::generate_background(manga_dir, &rect_data, &output_dir, width, height)?;
    // Ok(res)

    // 保证输出目录存在
    std::fs::create_dir_all(&output_dir)
        .context(format!("创建目录 {} 失败", output_dir.display()))?;
    // 收集尺寸符合width和height的图片的路径
    let image_paths = create_image_paths(manga_dir, width, height);
    // 用于记录是否找到了黑色背景和白色背景的水印图片
    let black_status: Mutex<Option<()>> = Mutex::new(None);
    let white_status: Mutex<Option<()>> = Mutex::new(None);
    let black_found = || black_status.lock_or_panic().is_some();
    let white_found = || white_status.lock_or_panic().is_some();
    // 并发遍历image_paths
    let image_paths = image_paths.par_iter();
    image_paths.try_for_each(|path| -> anyhow::Result<()> {
        // 如果black_path和white_path都已经找到了，则直接跳过
        if black_found() && white_found() {
            return Ok(());
        }
        let mut img = image::open(path)
            .context(format!("打开图片 {} 失败", path.display()))?
            .to_rgb8();
        let (left, top) = (rect_data.left, rect_data.top);
        let (right, bottom) = (rect_data.right, rect_data.bottom);
        // 检查图片是否满足黑色或白色背景的条件
        let Some(is_black) = is_black_background(&img, &rect_data) else {
            return Ok(());
        };
        // 获取左上角的颜色
        let color = *img.get_pixel(left, top);
        // 把截图区域外的像素点设置为左上角的颜色
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x < left || x > right || y < top || y > bottom {
                *pixel = color;
            }
        }
        let filename = if is_black { "black.png" } else { "white.png" };
        let output_path = output_dir.join(filename);
        // 保存黑色背景或白色背景的水印图片
        let mut background_path = if is_black {
            black_status.lock_or_panic()
        } else {
            white_status.lock_or_panic()
        };
        // 如果background_path是None，则把output_path赋值给background_path，并保存图片
        if background_path.is_none() {
            *background_path = Some(());
            // 因为save是耗时操作，所以在这里手动释放锁
            drop(background_path);
            img.save(&output_path)
                .context(format!("保存图片 {} 失败", output_path.display()))?;
        }
        Ok(())
    })?;
    let mut res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    };
    if !black_found() {
        res.code = -1;
        res.msg += format!("找不到尺寸为({width}x{height})的黑色背景水印图\n").as_str();
    };
    if !white_found() {
        res.code = -1;
        res.msg += format!("找不到尺寸为({width}x{height})的白色背景水印图\n").as_str();
    };
    Ok(res)
}

/// 遍历`manga_dir`目录下的所有jpg文件，收集尺寸符合`width`和`height`的图片的路径
#[allow(clippy::cast_possible_truncation)]
fn create_image_paths(manga_dir: &str, width: u32, height: u32) -> Vec<PathBuf> {
    let image_paths: Vec<PathBuf> = WalkDir::new(PathBuf::from_slash(manga_dir))
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
            let size = imagesize::size(&path).ok()?;
            if size.width as u32 == width && size.height as u32 == height {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    image_paths
}

/// 检查图片`img`是否满足黑色背景的条件，如果返回`None`则表示既不满足黑色背景的条件也不满足白色背景的条件
#[allow(clippy::cast_precision_loss)]
fn is_black_background(img: &RgbImage, rect_data: &RectData) -> Option<bool> {
    let (left, top) = (rect_data.left, rect_data.top);
    let (right, bottom) = (rect_data.right, rect_data.bottom);
    let inside_rect = |x: u32, y: u32| x >= left && x <= right && y >= top && y <= bottom;
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
    // 统计rect_data区域内color颜色的像素点数量
    let color_count = img
        .enumerate_pixels()
        .filter(|(x, y, &pixel)| inside_rect(*x, *y) && pixel == color)
        .count();
    // 如果rect_data区域内的像素点数量大于总数的90%，则返回None
    if color_count as f32 / ((right - left + 1) * (bottom - top + 1)) as f32 > 0.9 {
        return None;
    }
    // 如果color所有通道的值都小于25，则认为是黑色背景
    let is_black = r <= 25;
    // 如果color所有通道的值都大于230，并且截图区域内的通道值都大于100(小于100一般是页码)，则认为是白色背景
    let is_white = r >= 230
        && img
            .enumerate_pixels()
            .filter(|(x, y, _)| inside_rect(*x, *y)) //矩形区域内的像素
            .all(|(_, _, pixel)| pixel.0[0] > 100); // 通道值大于100
    match (is_black, is_white) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}
