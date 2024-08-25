use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::sync::{mpsc, Semaphore};
use tokio::sync::mpsc::Receiver;

use crate::config::Config;
use crate::events;
use crate::responses::{BiliResponse, EpisodeData, ImageIndexData, ImageTokenData};
use crate::utils::filename_filter;

pub struct DownloadManager {
    sender: mpsc::Sender<u32>,
}

impl DownloadManager {
    pub fn new(app: AppHandle) -> Self {
        let (sender, receiver) = mpsc::channel::<u32>(32);
        tokio::task::spawn(receiver_loop(app, receiver));
        DownloadManager { sender }
    }

    pub async fn submit_episode(&self, ep_id: u32) -> anyhow::Result<()> {
        Ok(self.sender.send(ep_id).await?)
    }
}

async fn receiver_loop(app: AppHandle, mut receiver: Receiver<u32>) {
    let ep_sem = Arc::new(Semaphore::new(16));
    let img_sem = Arc::new(Semaphore::new(50));
    while let Some(ep_id) = receiver.recv().await {
        let app = app.clone();
        let ep_sem = ep_sem.clone();
        let img_sem = img_sem.clone();
        tokio::task::spawn(process_episode(app, ep_id, ep_sem, img_sem));
    }
}

#[allow(clippy::cast_possible_truncation)]
async fn process_episode(
    app: AppHandle,
    ep_id: u32,
    ep_sem: Arc<Semaphore>,
    img_sem: Arc<Semaphore>,
) -> anyhow::Result<()> {
    emit_pending_event(&app, ep_id);
    let _permit = ep_sem.acquire().await?;

    let config = Config::load(&app).map_err(anyhow::Error::from)?;
    let cookie = format!("SESSDATA={}", config.bili_cookie);

    let episode_data = get_episode_data(ep_id, &cookie).await?;
    let image_index_data = get_image_index_data(ep_id, &cookie).await?;
    let image_token_data = get_image_token_data(&image_index_data, &cookie).await?;

    let download_dir = get_download_dir(&app, &episode_data)?;
    let current = Arc::new(AtomicU32::new(0));
    let urls: Vec<String> = image_token_data
        .into_iter()
        .map(|data| (data.url, data.token))
        .map(|(url, token)| format!("{url}?token={token}"))
        .collect();

    emit_start_event(&app, ep_id, urls.len() as u32);
    for (i, url) in urls.iter().enumerate() {
        let save_path = download_dir.join(format!("{i:03}.jpg"));

        let app = app.clone();
        let img_sem = img_sem.clone();
        let url = url.clone();
        let current = current.clone();
        tokio::task::spawn(async move {
            if let Err(err) = download_image(url.clone(), save_path, img_sem).await {
                let err_msg = format!("下载图片失败: {err}");
                emit_error_event(&app, ep_id, url, err_msg);
            } else {
                let current = current.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                emit_success_event(&app, ep_id, url, current);
            }
        });
    }

    emit_end_event(&app, ep_id);

    Ok(())
}

fn emit_error_event(app: &AppHandle, ep_id: u32, url: String, err_msg: String) {
    let payload = events::DownloadImageErrorEventPayload {
        ep_id,
        url,
        err_msg,
    };
    let event = events::DownloadImageErrorEvent(payload);
    let _ = event.emit(app);
}

fn get_download_dir(app: &AppHandle, episode_data: &EpisodeData) -> anyhow::Result<PathBuf> {
    let title = filename_filter(&episode_data.title);
    let short_title = filename_filter(&episode_data.short_title);
    let ep_title = if title == short_title {
        title
    } else {
        format!("{short_title} {title}")
    };
    let download_dir = app
        .path()
        .resource_dir()?
        .join("漫画下载")
        .join(&episode_data.comic_title)
        .join(ep_title.trim());
    Ok(download_dir)
}

async fn download_image(
    url: String,
    save_path: PathBuf,
    img_sem: Arc<Semaphore>,
) -> anyhow::Result<()> {
    let _permit = img_sem.acquire().await?;

    let http_res = reqwest::get(&url).await?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await?;
        let err = anyhow!("下载图片 {url} 失败，预料之外的状态码: {text}");
        return Err(err);
    }

    let image_data = http_res.bytes().await?;

    if let Some(parent) = save_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(save_path, image_data).await?;
    Ok(())
}

async fn get_image_index_data(ep_id: u32, cookie: &str) -> anyhow::Result<ImageIndexData> {
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("cookie", cookie),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        headers.insert(key, HeaderValue::from_str(value)?);
    }

    let payload = json!({"ep_id": ep_id});

    let http_res = reqwest::Client::new()
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/GetImageIndex")
        .query(&[("device", "pc"), ("platform", "web")])
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await?;
        let err = anyhow!("获取章节 {ep_id} 的ImageIndexData失败，预料之外的错误: {text}");
        return Err(err);
    }

    let bili_res: BiliResponse = http_res.json().await?;
    if bili_res.code != 0 {
        let err = anyhow!("获取章节 {ep_id} 的ImageIndexData失败，预料之外的code: {bili_res:?}");
        return Err(err);
    }
    let Some(data) = bili_res.data else {
        let err = anyhow!("获取章节 {ep_id} 的ImageIndexData失败，data字段不存在: {bili_res:?}");
        return Err(err);
    };

    let data: ImageIndexData = serde_json::from_value(data)?;
    Ok(data)
}

async fn get_image_token_data(
    image_index_data: &ImageIndexData,
    cookie: &str,
) -> anyhow::Result<ImageTokenData> {
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("cookie", cookie),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        headers.insert(key, HeaderValue::from_str(value)?);
    }

    let urls: Vec<String> = image_index_data
        .images
        .iter()
        .map(|img| img.path.clone())
        .collect();
    let urls_str = serde_json::to_string(&urls)?;
    let payload = json!({"urls": urls_str});

    let http_res = reqwest::Client::new()
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/ImageToken")
        .query(&[("device", "pc"), ("platform", "web")])
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await?;
        let err = anyhow!("获取ImageTokenData失败，预料之外的状态码: {text}");
        return Err(err);
    }

    let bili_res: BiliResponse = http_res.json().await?;
    if bili_res.code != 0 {
        let err = anyhow!("获取ImageTokenData失败，预料之外的code: {bili_res:?}");
        return Err(anyhow!(err));
    }
    let Some(data) = bili_res.data else {
        let err = anyhow!("获取ImageTokenData失败，data字段不存在: {bili_res:?}");
        return Err(anyhow!(err));
    };

    let data: ImageTokenData = serde_json::from_value(data)?;
    Ok(data)
}

async fn get_episode_data(ep_id: u32, cookie: &str) -> anyhow::Result<EpisodeData> {
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("cookie", cookie),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        headers.insert(key, HeaderValue::from_str(value)?);
    }

    let payload = json!({"id": ep_id});

    let http_res = reqwest::Client::new()
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/GetEpisode")
        .query(&[("device", "pc"), ("platform", "web")])
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await?;
        let err = anyhow!("获取章节 {ep_id} 的数据失败，预料之外的状态码: {text}");
        return Err(err);
    }

    let bili_res: BiliResponse = http_res.json().await?;
    if bili_res.code != 0 {
        let err = anyhow!("获取章节 {ep_id} 的数据失败，预料之外的code: {bili_res:?}");
        return Err(err);
    }
    let Some(data) = bili_res.data else {
        let err = anyhow!("获取章节 {ep_id} 的数据失败，data字段不存在: {bili_res:?}");
        return Err(err);
    };

    let data: EpisodeData = serde_json::from_value(data)?;
    Ok(data)
}

fn emit_start_event(app: &AppHandle, ep_id: u32, total: u32) {
    let payload = events::DownloadEpisodeStartEventPayload { ep_id, total };
    let event = events::DownloadEpisodeStartEvent(payload);
    let _ = event.emit(app);
}

fn emit_pending_event(app: &AppHandle, ep_id: u32) {
    let payload = events::DownloadEpisodePendingEventPayload { ep_id };
    let event = events::DownloadEpisodePendingEvent(payload);
    let _ = event.emit(app);
}

fn emit_success_event(app: &AppHandle, ep_id: u32, url: String, current: u32) {
    let payload = events::DownloadImageSuccessEventPayload {
        ep_id,
        url,
        current,
    };
    let event = events::DownloadImageSuccessEvent(payload);
    let _ = event.emit(app);
}

fn emit_end_event(app: &AppHandle, ep_id: u32) {
    let payload = events::DownloadEpisodeEndEventPayload { ep_id };
    let event = events::DownloadEpisodeEndEvent(payload);
    let _ = event.emit(app);
}
