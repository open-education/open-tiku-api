use crate::api::resp::class::ClassInfoResp;
use crate::api::resp::class_student::ClassStudentResp;
use serde::Serialize;

#[derive(Serialize)]
pub struct HomeworkInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "batchNo"))]
    pub batch_no: i32,
    #[serde(rename(serialize = "homeworkId"))]
    pub homework_id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(serialize = "classInfo"))]
    pub class_info: ClassInfoResp,
    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64,
    pub title: String,
    pub remark: String,
    pub students: Vec<ClassStudentResp>,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
}

#[derive(Serialize)]
pub struct HomeworkListResp {
    pub list: Vec<HomeworkInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
