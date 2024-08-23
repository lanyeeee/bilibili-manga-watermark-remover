use anyhow::anyhow;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;

use crate::errors::CommandResult;
use crate::responses::{BiliResponse, CookieStatusData};
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
pub async fn get_bili_cookie_status_data(
    bili_cookie: &str,
) -> CommandResult<CommandResponse<CookieStatusData>> {
    let cookie = format!("SESSDATA={bili_cookie}");
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("cookie", &cookie),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        let value = HeaderValue::from_str(value).map_err(anyhow::Error::from)?;
        headers.insert(key, value);
    }

    let http_res = reqwest::Client::new()
        .get("https://api.bilibili.com/x/web-interface/nav")
        .headers(headers)
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await.map_err(anyhow::Error::from)?;
        return Err(anyhow!("检查cookie状态失败，预料之外的状态码: {text}").into());
    }

    let bili_res: BiliResponse = http_res.json().await.map_err(anyhow::Error::from)?;
    if bili_res.code != 0 {
        return Err(anyhow!("检查cookie状态失败，预料之外的code: {bili_res:?}").into());
    }
    let Some(data) = bili_res.data else {
        return Err(anyhow!("检查cookie状态失败，data字段不存在: {bili_res:?}").into());
    };

    let data: CookieStatusData = serde_json::from_value(data).map_err(anyhow::Error::from)?;
    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data,
    };
    Ok(res)
}
