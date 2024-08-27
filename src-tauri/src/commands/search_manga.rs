use std::sync::RwLock;

use anyhow::anyhow;
use serde_json::json;
use tauri::http::{HeaderMap, HeaderValue, StatusCode};
use tauri::State;

use crate::config::Config;
use crate::errors::CommandResult;
use crate::extensions::IgnoreRwLockPoison;
use crate::responses::{BiliResponse, SearchData};
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
pub async fn search_manga(
    config: State<'_, RwLock<Config>>,
    keyword: &str,
) -> CommandResult<CommandResponse<SearchData>> {
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

    let payload = json!({
        "key_word": keyword,
        "page_num": 1,
        "page_size": 99
    });

    let http_res = reqwest::Client::new()
        .post("https://manga.bilibili.com/twirp/comic.v1.Comic/Search?device=pc&platform=web")
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await.map_err(anyhow::Error::from)?;
        return Err(anyhow!("搜索漫画失败，预料之外的状态码: {text}").into());
    }

    let bili_res: BiliResponse = http_res.json().await.map_err(anyhow::Error::from)?;
    if bili_res.code != 0 {
        return Err(anyhow!("搜索漫画失败，预料之外的code: {bili_res:?}").into());
    }
    let Some(data) = bili_res.data else {
        return Err(anyhow!("搜索漫画失败，data字段不存在: {bili_res:?}").into());
    };

    let search_data: SearchData = serde_json::from_value(data).map_err(anyhow::Error::from)?;
    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: search_data,
    };
    Ok(res)
}
