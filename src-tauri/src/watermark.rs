use crate::extensions::IgnoreLockPoison;
use crate::{events, types};
use anyhow::{anyhow, Context};
use image::{Rgb, RgbImage};
use path_slash::PathBufExt;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri_specta::Event;
use walkdir::{DirEntry, WalkDir};

/// 生成黑色背景和白色背景的水印图片
#[allow(clippy::cast_possible_truncation)]
pub fn generate_background(
    manga_dir: &str,
    rect_data: &types::RectData,
    output_dir: &Path,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    // 遍历manga_dir目录下的所有jpg文件，收集尺寸符合要求的图片的路径
    let image_paths: Vec<PathBuf> = WalkDir::new(PathBuf::from_slash(manga_dir))
        .max_depth(2) // 一般第一层目录是章节目录，第二层目录是图片文件
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
    // 用于记录是否找到了黑色背景和白色背景的水印图片
    let black_path: Mutex<Option<()>> = Mutex::new(None);
    let white_path: Mutex<Option<()>> = Mutex::new(None);
    // 并发遍历image_paths
    let image_paths = image_paths.par_iter();
    image_paths.try_for_each(|path| -> anyhow::Result<()> {
        // 如果black_path和white_path都已经找到了，则直接跳过
        if black_path.lock_or_panic().is_some() && white_path.lock_or_panic().is_some() {
            return Ok(());
        }
        let mut img = image::open(path)
            .context(format!("打开图片 {} 失败", path.display()))?
            .to_rgb8();
        let (left, top) = (rect_data.left, rect_data.top);
        let (right, bottom) = (rect_data.right, rect_data.bottom);
        // 检查图片是否满足黑色或白色背景的条件
        let Some(is_black) = is_black_background(&img, rect_data) else {
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
            black_path.lock_or_panic()
        } else {
            white_path.lock_or_panic()
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
    // 检查是否找到了黑色背景和白色背景的水印图片
    if black_path.lock_or_panic().is_none() {
        return Err(anyhow!(
            "在漫画目录 {manga_dir} 下找不到合适的黑色背景水印图({width}x{height})",
        ));
    }
    if white_path.lock_or_panic().is_none() {
        return Err(anyhow!(
            "在漫画目录 {manga_dir} 下找不到合适的白色背景水印图({width}x{height})",
        ));
    }

    Ok(())
}

/// 移除`manga_dir`目录下所有图片的水印，并保存到`output_dir`目录
#[allow(clippy::cast_possible_truncation)]
pub fn remove(
    app: &tauri::AppHandle,
    manga_dir: &str,
    output_dir: &str,
    black_data: &types::JpgImageData,
    white_data: &types::JpgImageData,
) -> anyhow::Result<()> {
    let manga_dir = PathBuf::from_slash(manga_dir);
    let manga_dir_without_name = manga_dir
        .parent()
        .ok_or(anyhow!("漫画目录 {} 的父目录不存在", manga_dir.display()))?;
    let output_dir = PathBuf::from_slash(output_dir);
    let black = black_data
        .to_image()
        .context(format!("黑色背景水印图 {} 转换失败", black_data.info.path))?
        .to_rgb8();
    let white = white_data
        .to_image()
        .context(format!("白色背景水印图 {} 转换失败", white_data.info.path))?
        .to_rgb8();
    if black.dimensions() != white.dimensions() {
        return Err(anyhow!(
            "黑色背景和白色背景水印图的尺寸不一致，黑色背景水印图的尺寸是 ({}x{})，白色背景水印图的尺寸是 ({}x{})",
            black.width(),
            black.height(),
            white.width(),
            white.height(),
        ));
    }
    // 构建一个HashMap，key是目录的路径，value是该目录下的所有jpg文件的路径
    let mut dir_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    // 遍历manga_dir目录下的所有文件和子目录
    for entry in WalkDir::new(&manga_dir).into_iter().filter_map(Result::ok) {
        let entry: DirEntry = entry;
        let path = entry.into_path();
        // 如果是文件且是jpg文件则添加到directory_map中
        if path.is_file() && path.extension().map_or(false, |e| e == "jpg") {
            if let Some(parent) = path.parent() {
                dir_map.entry(parent.to_path_buf()).or_default().push(path);
            }
        }
    }
    let dir_progress: HashMap<&PathBuf, (u32, u32)> = dir_map
        .keys()
        .map(|dir| {
            let total = dir_map[dir].len() as u32;
            // 发送RemoveWatermarkStartEvent事件
            let payload = events::RemoveWatermarkStartEventPayload {
                dir_path: dir.display().to_string(),
                total,
            };
            let event = events::RemoveWatermarkStartEvent(payload);
            event.emit(app).map_err(anyhow::Error::from)?;

            Ok((dir, (0, total)))
        })
        .collect::<anyhow::Result<HashMap<&PathBuf, (u32, u32)>>>()?;

    let dir_progress = Mutex::new(dir_progress);
    // 使用rayon的并行迭代器，并行处理每个目录
    let dir_map = dir_map.par_iter();
    dir_map.try_for_each(|entry| -> anyhow::Result<()> {
        let (dir, img_paths) = entry;
        // 使用rayon的并行迭代器，并行处理每个目录下的图片
        let img_paths = img_paths.par_iter();
        img_paths.try_for_each(|img_path| -> anyhow::Result<()> {
            // 获取相对路径(漫画名/章节名/图片名)
            let relative_path = img_path
                .strip_prefix(manga_dir_without_name)
                .context(format!(
                    "{} 不是 {} 的父目录",
                    manga_dir_without_name.display(),
                    img_path.display()
                ))?;
            // 构建输出图片的路径(输出目录/漫画名/章节名/图片名)
            let out_image_path = output_dir.join(relative_path);
            // 打开输入图片
            let mut img = image::open(img_path)
                .context(format!("打开图片 {} 失败", img_path.display()))?
                .to_rgb8();
            // 去除水印
            remove_image_watermark(&white, &black, &mut img);
            // 保存去除水印后的图片(无论是否成功去除水印都会保存)
            save_jpg_image(&img, &out_image_path)
                .context(format!("保存图片 {} 失败", out_image_path.display()))?;
            // 更新目录的进度
            let (current, total) = {
                let mut dir_progress = dir_progress.lock_or_panic();
                let (current, total) = dir_progress
                    .get_mut(dir)
                    .ok_or(anyhow!("目录 {} 的进度不存在", dir.display()))?;
                *current += 1;
                (*current, *total)
            };
            // 发送RemoveWatermarkSuccessEvent事件
            let payload = events::RemoveWatermarkSuccessEventPayload {
                dir_path: dir.display().to_string(),
                img_path: out_image_path.display().to_string(),
                current,
            };
            let event = events::RemoveWatermarkSuccessEvent(payload);
            event.emit(app)?;
            // 如果当前图片是目录下的最后一张图片，则发送RemoveWatermarkEndEvent事件
            if current == total {
                let payload = events::RemoveWatermarkEndEventPayload {
                    dir_path: dir.display().to_string(),
                };
                let event = events::RemoveWatermarkEndEvent(payload);
                event.emit(app)?;
            }

            Ok(())
        })?;
        Ok(())
    })?;

    Ok(())
}

/// 检查图片`img`是否满足黑色背景的条件，如果返回`None`则表示既不满足黑色背景的条件也不满足白色背景的条件
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

/// 去除`img`的水印
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_sign_loss)]
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
        // 计算去除水印后的像素点值，将f32转换为u8自带clamp功能
        let watermark_removed_pixel = Rgb([
            ((img_r as f32 - black_r as f32) / ((white_r - black_r) as f32 / 255.0)) as u8,
            ((img_g as f32 - black_g as f32) / ((white_g - black_g) as f32 / 255.0)) as u8,
            ((img_b as f32 - black_b as f32) / ((white_b - black_b) as f32 / 255.0)) as u8,
        ]);
        // 将去除水印后的像素点值写入到图片缓冲区中
        *img_pixel = watermark_removed_pixel;
    }
}

/// 保存jpg图片`img`到指定路径`path`
#[allow(clippy::cast_possible_truncation)]
fn save_jpg_image(img: &RgbImage, path: &Path) -> anyhow::Result<()> {
    // 保证输出目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(format!("创建目录 {} 失败", parent.display()))?;
    }
    // 保存去除水印后的图片，使用jpeg_encoder库的Encoder，效率更高
    let encoder = jpeg_encoder::Encoder::new_file(path, 97)?;
    encoder
        .encode(
            img.as_raw(),
            img.width() as u16,
            img.height() as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .context(format!("编码图片 {} 失败", path.display()))?;
    Ok(())
}
