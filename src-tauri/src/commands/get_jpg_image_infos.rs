use std::path::PathBuf;

use walkdir::WalkDir;

use crate::types::JpgImageInfo;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
pub fn get_jpg_image_infos(manga_dir: &str) -> Vec<JpgImageInfo> {
    // 用于存储jpg图片的信息
    let mut jpg_image_infos = vec![];
    // 遍历漫画目录下的所有文件，获取jpg图片的信息
    WalkDir::new(PathBuf::from(manga_dir))
        .max_depth(2) //  一般第一层目录是章节目录，第二层目录是图片文件
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
            let size = image::image_dimensions(&path).ok()?;
            Some((path, size))
        })
        .for_each(|(path, size)| {
            jpg_image_infos.push(JpgImageInfo {
                width: size.0,
                height: size.1,
                path,
            });
        });

    jpg_image_infos
}
