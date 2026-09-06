use crate::model::other_dict::TextbookDict;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextbookDictResp {
    pub id: i32,
    pub textbook_id: i32,
    pub type_code: String,
    pub item_value: String,
    pub sort_order: i32,
    pub is_select: bool,
}

impl From<TextbookDict> for TextbookDictResp {
    fn from(row: TextbookDict) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            textbook_id: row.textbook_id,
            type_code: row.type_code,
            item_value: row.item_value,
            sort_order: row.sort_order,
            is_select: row.is_select,
        }
    }
}
