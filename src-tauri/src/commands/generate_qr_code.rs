use std::io::Cursor;

use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use image::Rgb;
use qrcode::QrCode;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;

use crate::errors::CommandResult;
use crate::responses::{BiliResponse, GenerateQrCodeData};
use crate::types::{CommandResponse, QrCodeData};

#[tauri::command(async)]
#[specta::specta]
pub async fn generate_qr_code() -> CommandResult<CommandResponse<QrCodeData>> {
    let headers_vec = [
        ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ("origin", "https://manga.bilibili.com"),
    ];
    let mut headers = HeaderMap::new();
    for (key, value) in headers_vec {
        headers.insert(
            key,
            HeaderValue::from_str(value).map_err(anyhow::Error::from)?,
        );
    }

    let http_res = reqwest::Client::new()
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .headers(headers)
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await.map_err(anyhow::Error::from)?;
        return Err(anyhow!("生成二维码失败，预料之外的状态码: {text}").into());
    }

    let bili_res: BiliResponse = http_res.json().await.map_err(anyhow::Error::from)?;
    if bili_res.code != 0 {
        return Err(anyhow!("生成二维码失败，预料之外的code: {bili_res:?}").into());
    }
    let Some(data) = bili_res.data else {
        return Err(anyhow!("生成二维码失败，data字段不存在: {bili_res:?}").into());
    };

    let generate_qr_code_data: GenerateQrCodeData =
        serde_json::from_value(data).map_err(anyhow::Error::from)?;
    // 生成二维码
    let qr_code = QrCode::new(generate_qr_code_data.url).map_err(anyhow::Error::from)?;
    let img = qr_code.render::<Rgb<u8>>().build();
    let mut img_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_data), image::ImageFormat::Jpeg)
        .map_err(anyhow::Error::from)?;
    let base64 = general_purpose::STANDARD.encode(img_data);
    let qr_code_data = QrCodeData {
        base64,
        qrcode_key: generate_qr_code_data.qrcode_key,
    };
    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: qr_code_data,
    };
    Ok(res)
}
