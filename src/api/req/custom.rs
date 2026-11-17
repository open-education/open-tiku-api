use crate::model::custom::CustomTextbook;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddReq {
    pub id: Option<i64>,
    pub user_source: i16,
    pub custom_type: i16,
    pub textbook: Option<CustomTextbook>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomListReq {
    pub page_source: i16,
    pub user_source: i16,
}
