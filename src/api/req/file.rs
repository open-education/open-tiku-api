use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteFileReq {
    #[serde(rename(deserialize = "isImage"))]
    pub is_image: bool,
    pub filename: String,
}
