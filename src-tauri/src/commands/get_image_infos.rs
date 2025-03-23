use std::path::PathBuf;

use walkdir::WalkDir;

use crate::{extensions::PathIsImage, types::ImageInfo};

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
pub fn get_image_infos(manga_dir: &str) -> Vec<ImageInfo> {
    // 用于存储图片的信息
    let mut image_infos = vec![];
    // 遍历漫画目录下的所有文件，获取图片的信息
    WalkDir::new(PathBuf::from(manga_dir))
        .max_depth(2) //  一般第一层目录是章节目录，第二层目录是图片文件
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.into_path();
            if !path.is_file() || !path.is_image() {
                return None;
            }
            let size = image::image_dimensions(&path).ok()?;
            Some((path, size))
        })
        .for_each(|(path, size)| {
            image_infos.push(ImageInfo {
                width: size.0,
                height: size.1,
                path,
            });
        });

    image_infos
}
