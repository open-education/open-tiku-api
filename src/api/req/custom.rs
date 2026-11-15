use crate::model::custom::CustomTextbook;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomReq {
    pub id: Option<i64>,
    pub custom_type: i16,
    pub textbook: Option<CustomTextbook>,
}
