use crate::model::textbook::Textbook;
use serde::Serialize;

#[derive(Serialize)]
pub struct TextbookResp {
    pub id: i32,
    #[serde(rename(serialize = "pathType"))]
    pub path_type: String,
    #[serde(rename(serialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32, // 默认为 0
    #[serde(rename(serialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    pub path: String,
    #[serde(rename(serialize = "tableName"))]
    pub table_name: Option<String>,
    pub children: Option<Vec<TextbookResp>>,
}

impl From<Textbook> for TextbookResp {
    fn from(row: Textbook) -> Self {
        Self {
            id: row.id,
            path_type: row.path_type,
            parent_id: row.parent_id,
            label: row.label,
            key: row.key,
            sort_order: row.sort_order,
            path_depth: row.path_depth,
            path: row.path,
            table_name: Some("textbook".to_string()),
            children: None,
        }
    }
}
