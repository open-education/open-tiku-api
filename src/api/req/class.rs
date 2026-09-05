use serde::Deserialize;

// 班级添加
#[derive(Deserialize)]
pub struct ClassInfoReq {
    pub id: Option<i64>,
    pub year: String,
    pub grade: Option<String>,
    pub semester: Option<String>,
    pub label: String,
    pub email: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i16,
    pub remark: String,
}
