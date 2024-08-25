use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeData {
    pub title: String,
    #[serde(rename = "comic_id")]
    pub comic_id: i64,
    #[serde(rename = "short_title")]
    pub short_title: String,
    #[serde(rename = "comic_title")]
    pub comic_title: String,
}
