use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommonEditStatusReq {
    pub id: i64,
    pub status: i16,
    #[serde(rename(deserialize = "rejectReason"))]
    pub reject_reason: Option<String>,
}
