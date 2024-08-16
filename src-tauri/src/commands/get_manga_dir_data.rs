use std::collections::HashMap;
use std::path::PathBuf;

use path_slash::PathBufExt;
use tauri::AppHandle;
use walkdir::WalkDir;

use crate::commands::open_image;
use crate::errors::CommandResult;
use crate::types::{CommandResponse, MangaDirData};
use crate::utils;

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::needless_pass_by_value)]
pub fn get_manga_dir_data(
    app: AppHandle,
    manga_dir: &str,
) -> CommandResult<CommandResponse<Vec<MangaDirData>>> {
    // 用于存储不同尺寸的图片的数量
    let mut size_count: HashMap<(u32, u32), u32> = HashMap::new();
    // 遍历漫画目录下的所有文件，统计不同尺寸的图片的数量
    WalkDir::new(PathBuf::from_slash(manga_dir))
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
            imagesize::size(&path).ok()
        })
        .for_each(|size| {
            let key = (size.width as u32, size.height as u32);
            let count = size_count.entry(key).or_insert(0);
            *count += 1;
        });
    // 将统计结果转换为Vec<MangaDirData>
    let mut manga_dir_data: Vec<MangaDirData> = size_count
        .into_iter()
        .map(|((width, height), count)| MangaDirData {
            width,
            height,
            count,
            black_background: None,
            white_background: None,
        })
        .collect();
    // 以count降序排序
    manga_dir_data.sort_by(|a, b| b.count.cmp(&a.count));
    // 获取背景水印图的数据
    for dir_data in &mut manga_dir_data {
        let width = dir_data.width;
        let height = dir_data.height;
        let background_dir = utils::get_background_dir_abs_path(&app, manga_dir, width, height)?;
        let black_background_path = background_dir.join("black.png");
        let white_background_path = background_dir.join("white.png");
        if black_background_path.exists() {
            let black_background_path = black_background_path.display().to_string();
            let black_background = open_image(black_background_path).map(|res| Some(res.data))?;
            dir_data.black_background = black_background;
        }
        if white_background_path.exists() {
            let white_background_path = white_background_path.display().to_string();
            let white_background = open_image(white_background_path).map(|res| Some(res.data))?;
            dir_data.white_background = white_background;
        }
    }
    // 返回结果
    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: manga_dir_data,
    };

    Ok(res)
}
