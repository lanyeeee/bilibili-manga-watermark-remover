use anyhow::anyhow;
use reqwest::StatusCode;

use crate::errors::CommandResult;
use crate::responses::{BiliResponse, QrCodeStatusData};
use crate::types::CommandResponse;

#[tauri::command(async)]
#[specta::specta]
pub async fn get_qr_code_status_data(
    qrcode_key: &str,
) -> CommandResult<CommandResponse<QrCodeStatusData>> {
    let http_res = reqwest::Client::new()
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
        .query(&[("qrcode_key", qrcode_key)])
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    let status = http_res.status();
    if status != StatusCode::OK {
        let text = http_res.text().await.map_err(anyhow::Error::from)?;
        return Err(anyhow!("获取二维码状态失败，预料之外的状态码: {text}").into());
    }

    let bili_res: BiliResponse = http_res.json().await.map_err(anyhow::Error::from)?;
    if bili_res.code != 0 {
        return Err(anyhow!("获取二维码状态失败，预料之外的code: {bili_res:?}").into());
    }
    let Some(data) = bili_res.data else {
        return Err(anyhow!("获取二维码状态失败，data字段不存在: {bili_res:?}").into());
    };

    let qr_code_status_data: QrCodeStatusData =
        serde_json::from_value(data).map_err(anyhow::Error::from)?;

    let res = CommandResponse {
        code: 0,
        msg: String::new(),
        data: qr_code_status_data,
    };

    Ok(res)
}
