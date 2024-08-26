use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliResponse {
    pub code: i64,
    #[serde(alias = "message")]
    pub msg: String,
    pub data: Option<serde_json::Value>,
}
