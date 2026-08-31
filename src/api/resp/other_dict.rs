use crate::model::other_dict::TextbookDict;
use serde::Serialize;

#[derive(Serialize)]
pub struct TextbookDictResp {
    pub id: i32,
    #[serde(rename(serialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(serialize = "typeCode",))]
    pub type_code: String,
    #[serde(rename(serialize = "itemValue"))]
    pub item_value: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(serialize = "isSelect"))]
    pub is_select: bool,
}

impl From<TextbookDict> for TextbookDictResp {
    fn from(row: TextbookDict) -> Self {
        Self {
            id: row.id,
            textbook_id: row.textbook_id,
            type_code: row.type_code,
            item_value: row.item_value,
            sort_order: row.sort_order,
            is_select: row.is_select,
        }
    }
}
