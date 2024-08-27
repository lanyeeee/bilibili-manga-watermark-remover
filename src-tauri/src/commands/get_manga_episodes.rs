use std::sync::RwLock;

use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde_json::json;
use tauri::{AppHandle, Manager, State};

use crate::config::Config;
use crate::errors::CommandResult;
use crate::extensions::IgnoreRwLockPoison;
use crate::responses::{BiliResponse, EpList, MangaData};
use crate::types::{CommandResponse, Episode};
use crate::utils::filename_filter;

#[tauri::command(async)]
#[specta::specta]
pub async fn get_manga_episodes(
    app: AppHandle,
    config: State<'_, RwLock<Config>>,
    id: i32,
) -> CommandResult<CommandResponse<Vec<Episode>>> {
    let cookie = config.read_or_panic().get_cookie();
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("cookie", &cookie),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        let value = HeaderValue::from_str(value).map_err(anyhow::Error::from)?;
        headers.insert(key, value);
    }

    let payload = json!({"comic_id": id});

    let http_res = reqwest::Client::new()
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/ComicDetail?device=pc&platform=web")
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await.map_err(anyhow::Error::from)?;
        return match status {
            StatusCode::BAD_REQUEST => Err(anyhow!("漫画不存在: {text}").into()),
            _ => Err(anyhow!("获取漫画详情失败，预料之外的状态码: {text}").into()),
        };
    }

    let bili_res: BiliResponse = http_res.json().await.map_err(anyhow::Error::from)?;
    if bili_res.code != 0 {
        return Err(anyhow!("获取漫画详情失败，预料之外的code: {bili_res:?}").into());
    }
    let Some(data) = bili_res.data else {
        return Err(anyhow!("获取漫画详情失败，data字段不存在: {bili_res:?}").into());
    };
    let mut manga_data: MangaData = serde_json::from_value(data).map_err(anyhow::Error::from)?;
    manga_data.ep_list.reverse();

    let mut episodes = Vec::with_capacity(manga_data.ep_list.len());
    for ep in manga_data.ep_list {
        let ep_id = ep.id;
        let ep_title = get_ep_title(&ep);
        let comic_id = manga_data.id;
        let comic_title = manga_data.title.clone();
        let is_locked = ep.is_locked;
        let is_downloaded = get_is_downloaded(&app, &ep_title, &comic_title)?;
        let episode = Episode {
            ep_id,
            ep_title,
            comic_id,
            comic_title,
            is_locked,
            is_downloaded,
        };
        episodes.push(episode);
    }

    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: episodes,
    };

    Ok(res)
}

fn get_is_downloaded(app: &AppHandle, ep_title: &str, comic_title: &str) -> anyhow::Result<bool> {
    let download_dir = app
        .path()
        .resource_dir()?
        .join("漫画下载")
        .join(comic_title)
        .join(ep_title);
    let is_downloaded = download_dir.exists();
    Ok(is_downloaded)
}

fn get_ep_title(ep: &EpList) -> String {
    let title = filename_filter(&ep.title);
    let short_title = filename_filter(&ep.short_title);
    let ep_title = if title == short_title {
        title
    } else {
        format!("{short_title} {title}")
    };
    ep_title.trim().to_string()
}
