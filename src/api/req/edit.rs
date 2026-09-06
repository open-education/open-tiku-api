use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonEditStatusReq {
    pub id: i64,
    pub status: i16,
    pub reject_reason: Option<String>,
}
