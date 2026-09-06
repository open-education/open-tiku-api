use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileReq {
    pub is_image: bool,
    pub filename: String,
}
