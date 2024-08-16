use std::collections::HashMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{anyhow, Context};
use image::{Rgb, RgbImage};
use image::codecs::png::PngEncoder;
use path_slash::PathBufExt;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tauri::AppHandle;
use tauri_specta::Event;
use walkdir::WalkDir;

use crate::errors::CommandResult;
use crate::events;
use crate::extensions::IgnoreLockPoison;
use crate::types::{CommandResponse, ImageFormat, JpgImageData};

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn remove_watermark(
    app: AppHandle,
    manga_dir: &str,
    output_dir: &str,
    format: ImageFormat,
    optimize: bool,
    backgrounds_data: Vec<(JpgImageData, JpgImageData)>,
) -> CommandResult<CommandResponse<()>> {
    let manga_dir = PathBuf::from_slash(manga_dir);
    let manga_dir_without_name = manga_dir
        .parent()
        .ok_or(anyhow!("漫画目录 {} 的父目录不存在", manga_dir.display()))?;
    let output_dir = PathBuf::from_slash(output_dir);
    // (width, height) => (black, white)
    let backgrounds = create_backgrounds(&backgrounds_data)?;
    // dir => [img_path1, img_path2, ...]
    let dir_map = create_dir_map(&manga_dir);
    // dir => (current, total)
    let dir_progress = create_dir_progress(&app, &dir_map)?;
    // 使用Mutex包装dir_progress，用于并发更新目录的进度
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
            let (width, height) = (img.width(), img.height());
            if let Some((black, white)) = backgrounds.get(&(width, height)) {
                // 只有在backgrounds中找到了黑色背景和白色背景的水印图片才会去除水印
                remove_image_watermark(black, white, &mut img);
            }
            // 保存去除水印后的图片(无论是否成功去除水印都会保存)
            save_image(&img, &out_image_path, &format, optimize)
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
                dir_path: dir.clone(),
                img_path: out_image_path.clone(),
                current,
            };
            let event = events::RemoveWatermarkSuccessEvent(payload);
            event.emit(&app)?;
            // 如果当前图片是目录下的最后一张图片，则发送RemoveWatermarkEndEvent事件
            if current == total {
                let payload = events::RemoveWatermarkEndEventPayload {
                    dir_path: dir.clone(),
                };
                let event = events::RemoveWatermarkEndEvent(payload);
                event.emit(&app)?;
            }

            Ok(())
        })?;
        Ok(())
    })?;

    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: (),
    };
    Ok(res)
}

/// 构建一个`HashMap`，`key`是目录的路径，`value`是该目录下的所有jpg文件的路径
#[allow(clippy::cast_possible_truncation)]
fn create_dir_progress<'a>(
    app: &AppHandle,
    dir_map: &'a HashMap<PathBuf, Vec<PathBuf>>,
) -> anyhow::Result<HashMap<&'a PathBuf, (u32, u32)>> {
    let dir_progress: HashMap<&PathBuf, (u32, u32)> = dir_map
        .keys()
        .map(|dir| {
            let total = dir_map[dir].len() as u32;
            // 发送RemoveWatermarkStartEvent事件
            let payload = events::RemoveWatermarkStartEventPayload {
                dir_path: dir.clone(),
                total,
            };
            let event = events::RemoveWatermarkStartEvent(payload);
            event.emit(app).map_err(anyhow::Error::from)?;

            Ok((dir, (0, total)))
        })
        .collect::<anyhow::Result<HashMap<&PathBuf, (u32, u32)>>>()?;
    Ok(dir_progress)
}

/// 构建一个`HashMap`，`key`是目录的路径，`value`是该目录下的所有jpg文件的路径
fn create_dir_map(manga_dir: &PathBuf) -> HashMap<PathBuf, Vec<PathBuf>> {
    let mut dir_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    // 遍历manga_dir目录下的所有文件和子目录
    WalkDir::new(manga_dir)
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
            let parent = path.parent()?.to_path_buf();
            Some((path, parent))
        })
        .for_each(|(path, parent)| dir_map.entry(parent).or_default().push(path));
    dir_map
}

/// 构建一个`HashMap`，`key`是背景水印图的尺寸，`value`是黑色背景和白色背景水印图
fn create_backgrounds(
    backgrounds_data: &[(JpgImageData, JpgImageData)],
) -> anyhow::Result<HashMap<(u32, u32), (RgbImage, RgbImage)>> {
    let backgrounds = backgrounds_data
        .iter()
        .map(|(black_data, white_data)| {
            let black = black_data
                .to_image()
                .context(format!("黑色背景水印图 {} 转换失败", black_data.info.path.display()))?
                .to_rgb8();
            let white = white_data
                .to_image()
                .context(format!("白色背景水印图 {} 转换失败", white_data.info.path.display()))?
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
            Ok(((black.width(), black.height()), (black, white)))
        })
        .collect::<anyhow::Result<HashMap<(u32, u32), (RgbImage, RgbImage)>>>()?;
    Ok(backgrounds)
}

/// 去除`img`的水印
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_sign_loss)]
fn remove_image_watermark(black: &RgbImage, white: &RgbImage, img: &mut RgbImage) {
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

/// 保存图片`img`到指定路径`path`，`format`为图片格式，`optimize`为true时会检查图片是否为灰度图像，如果是则保存为luma8图片
#[allow(clippy::cast_possible_truncation)]
fn save_image(
    img: &RgbImage,
    path: &Path,
    format: &ImageFormat,
    optimize: bool,
) -> anyhow::Result<()> {
    // 保证输出目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(format!("创建目录 {} 失败", parent.display()))?;
    }

    match format {
        ImageFormat::Jpeg => {
            save_jpg_image(img, path, optimize)?;
        }
        ImageFormat::Png => {
            save_png_image(img, path, optimize)?;
        }
    }
    Ok(())
}

/// 保存jpg图片`img`到指定路径`path`, `optimize`为true时会检查图片是否为灰度图像，如果是则保存为luma8图片
#[allow(clippy::cast_possible_truncation)]
fn save_jpg_image(img: &RgbImage, path: &Path, optimize: bool) -> anyhow::Result<()> {
    let (width, height) = (img.width() as u16, img.height() as u16);
    // 保证后缀为jpg
    let path = path.with_extension("jpg");
    // 保存去除水印后的图片，使用jpeg_encoder库的Encoder处理jpg效率更高
    let encoder = jpeg_encoder::Encoder::new_file(&path, 95)?;
    if optimize && is_grey_image(img) {
        let luma = image::DynamicImage::ImageRgb8(img.clone()).into_luma8();
        encoder
            .encode(luma.as_raw(), width, height, jpeg_encoder::ColorType::Luma)
            .context(format!("编码luma8图片 {} 失败", path.display()))?;
    } else {
        encoder
            .encode(img.as_raw(), width, height, jpeg_encoder::ColorType::Rgb)
            .context(format!("编码rgb图片 {} 失败", path.display()))?;
    }
    Ok(())
}

/// 保存png图片`img`到指定路径`path`, `optimize`为true时会检查图片是否为灰度图像，如果是则保存为luma8图片
#[allow(clippy::cast_possible_truncation)]
fn save_png_image(img: &RgbImage, path: &Path, optimize: bool) -> anyhow::Result<()> {
    // 保证后缀为png
    let path = path.with_extension("png");
    let png_file = std::fs::File::create(&path)?;
    let buffered_file_writer = BufWriter::new(png_file);
    let encoder = PngEncoder::new(buffered_file_writer);
    if optimize && is_grey_image(img) {
        let luma = image::DynamicImage::ImageRgb8(img.clone()).into_luma8();
        luma.write_with_encoder(encoder)
            .context(format!("编码luma8图片 {} 失败", path.display()))?;
    } else {
        img.write_with_encoder(encoder)
            .context(format!("编码rgb图片 {} 失败", path.display()))?;
    }
    Ok(())
}

fn is_grey_image(img: &RgbImage) -> bool {
    img.pixels().all(|pixel| {
        let [r, g, b] = pixel.0;
        r == g && g == b
    })
}
