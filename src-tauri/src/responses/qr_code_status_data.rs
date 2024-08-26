use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::QrCodeStatus;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeStatusData {
    pub url: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    pub timestamp: i64,
    pub code: i64,
    pub message: String,
}

impl QrCodeStatusData {
    #[allow(dead_code)]
    pub fn status(&self) -> QrCodeStatus {
        match self.code {
            0 => {
                let sessdata = self.url.split("SESSDATA=").collect::<Vec<&str>>()[1]
                    .split('&')
                    .collect::<Vec<&str>>()[0]
                    .to_string();
                QrCodeStatus::Complete(sessdata)
            }
            86101 => QrCodeStatus::NotScan,
            86090 => QrCodeStatus::Scanning,
            86038 => QrCodeStatus::Invalid,
            _ => QrCodeStatus::Unknown,
        }
    }
}
