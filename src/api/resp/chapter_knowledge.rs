use crate::model::chapter_knowledge::ChapterKnowledge;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterKnowledgeResp {
    pub id: Option<i32>,
    pub chapter_id: i32,
    pub knowledge_id: i32,
}

impl From<ChapterKnowledge> for ChapterKnowledgeResp {
    fn from(row: ChapterKnowledge) -> Self {
        Self {
            id: Some(row.id),
            chapter_id: row.chapter_id,
            knowledge_id: row.knowledge_id,
        }
    }
}
