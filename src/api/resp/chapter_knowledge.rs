use crate::model::chapter_knowledge::ChapterKnowledge;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChapterKnowledgeResp {
    pub id: Option<i32>,
    #[serde(rename(serialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(serialize = "knowledgeId"))]
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
