use serde::{Deserialize, Serialize};

pub type ImageTokenData = Vec<UrlToken>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlToken {
    pub url: String,
    pub token: String,
}