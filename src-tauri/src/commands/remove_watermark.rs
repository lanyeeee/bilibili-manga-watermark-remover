use std::collections::HashMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use image::codecs::png::PngEncoder;
use image::{Rgb, RgbImage};
use parking_lot::Mutex;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tauri::AppHandle;
use tauri_specta::Event;
use walkdir::WalkDir;

use crate::errors::CommandResult;
use crate::events;
use crate::types::{ImageFormat, JpgImageData};

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
) -> CommandResult<()> {
    let manga_dir = PathBuf::from(manga_dir);
    let manga_dir_without_name = manga_dir
        .parent()
        .ok_or(anyhow!("漫画目录 {manga_dir:?} 的父目录不存在"))?;
    let output_dir = PathBuf::from(output_dir);
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
                    "{manga_dir_without_name:?} 不是 {img_path:?} 的父目录"
                ))?;
            // 构建输出图片的路径(输出目录/漫画名/章节名/图片名)
            let out_image_path = output_dir.join(relative_path);
            // 获取图片的尺寸
            let (width, height) = image::image_dimensions(img_path)
                .context(format!("获取图片 {img_path:?} 的尺寸失败"))?;
            if let Some((black, white)) = backgrounds.get(&(width, height)) {
                // 在backgrounds中找到了黑色背景和白色背景的水印图片，可以去除水印
                let mut img = image::open(img_path)
                    .context(format!("打开图片 {img_path:?} 失败"))?
                    .to_rgb8();

                remove_image_watermark(black, white, &mut img);

                save_image(&img, &out_image_path, &format, optimize)
                    .context(format!("保存图片 {out_image_path:?} 失败"))?;
            } else {
                // 否则，直接复制图片到输出目录
                if let Some(parent) = out_image_path.parent() {
                    // 保证输出目录存在
                    std::fs::create_dir_all(parent).context(format!("创建目录 {parent:?} 失败"))?;
                }
                std::fs::copy(img_path, &out_image_path)
                    .context(format!("复制图片 {img_path:?} 到 {out_image_path:?} 失败"))?;
            }
            // 更新目录的进度
            let (current, total) = {
                let mut dir_progress = dir_progress.lock();
                let (current, total) = dir_progress
                    .get_mut(dir)
                    .ok_or(anyhow!("目录 {dir:?} 的进度不存在"))?;
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

    Ok(())
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
                .context(format!("黑色背景水印图 {:?} 转换失败", black_data.info.path))?
                .to_rgb8();
            let white = white_data
                .to_image()
                .context(format!("白色背景水印图 {:?} 转换失败", white_data.info.path))?
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
    let [black_in_r, black_in_g, black_in_b] = black.get_pixel(0, 0).0.map(|x| x as f64);
    for (x, y, img_pixel) in img.enumerate_pixels_mut() {
        let [out_r, out_g, out_b] = img_pixel.0.map(|x| x as f64);
        let [black_out_r, black_out_g, black_out_b] = black.get_pixel(x, y).0.map(|x| x as f64);
        let [white_out_r, white_out_g, white_out_b] = white.get_pixel(x, y).0.map(|x| x as f64);

        let in_r = (out_r - black_out_r) / ((white_out_r - black_out_r) / 255.0) + black_in_r;
        let in_g = (out_g - black_out_g) / ((white_out_g - black_out_g) / 255.0) + black_in_g;
        let in_b = (out_b - black_out_b) / ((white_out_b - black_out_b) / 255.0) + black_in_b;
        // 将f64转换为u8自带clamp功能
        let watermark_removed_r = in_r.round() as u8;
        let watermark_removed_g = in_g.round() as u8;
        let watermark_removed_b = in_b.round() as u8;
        // 将去除水印后的像素点赋值给img
        *img_pixel = Rgb([
            watermark_removed_r,
            watermark_removed_g,
            watermark_removed_b,
        ]);
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
        std::fs::create_dir_all(parent).context(format!("创建目录 {parent:?} 失败"))?;
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
            .context(format!("编码luma8图片 {path:?} 失败"))?;
    } else {
        encoder
            .encode(img.as_raw(), width, height, jpeg_encoder::ColorType::Rgb)
            .context(format!("编码rgb图片 {path:?} 失败"))?;
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
            .context(format!("编码luma8图片 {path:?} 失败"))?;
    } else {
        img.write_with_encoder(encoder)
            .context(format!("编码rgb图片 {path:?} 失败"))?;
    }
    Ok(())
}

fn is_grey_image(img: &RgbImage) -> bool {
    img.pixels().all(|pixel| {
        let [r, g, b] = pixel.0;
        r == g && g == b
    })
}
