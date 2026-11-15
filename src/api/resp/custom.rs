use crate::model::custom::{Custom, CustomTextbook};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomResp {
    pub id: i64,
    pub custom_type: i16,
    pub textbook: CustomTextbook,
}

impl From<Custom> for CustomResp {
    fn from(row: Custom) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            custom_type: row.custom_type,
            textbook: row.textbook.0,
        }
    }
}
