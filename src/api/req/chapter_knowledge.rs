use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateChapterKnowledgeReq {
    #[serde(rename(deserialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

#[derive(Deserialize)]
pub struct RemoveChapterKnowledgeReq {
    pub id: i32,
    #[serde(rename(deserialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}
