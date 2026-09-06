use crate::api::resp::class::ClassInfoResp;
use crate::api::resp::class_student::ClassStudentResp;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeworkInfoResp {
    pub id: i64,
    pub batch_no: i32,
    pub homework_id: i64,
    pub paper_id: i64,
    pub class_id: i64,
    pub class_info: ClassInfoResp,
    pub author_id: i64,
    pub title: String,
    pub remark: String,
    pub students: Vec<ClassStudentResp>,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeworkListResp {
    pub list: Vec<HomeworkInfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}
