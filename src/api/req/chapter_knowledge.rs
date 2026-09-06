use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChapterKnowledgeReq {
    pub chapter_id: i32,
    pub knowledge_id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveChapterKnowledgeReq {
    pub id: i32,
    pub chapter_id: i32,
    pub knowledge_id: i32,
}
