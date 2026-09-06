use crate::model::class::Class;
use crate::util::local::to_local_datetime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ClassListReq {
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

// 班级信息返回
#[derive(Serialize, Default)]
pub struct ClassInfoResp {
    pub id: i64,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub label: String,
    pub email: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i16,
    pub remark: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

impl From<Class> for ClassInfoResp {
    fn from(row: Class) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            year: row.year,
            grade: row.grade,
            semester: row.semester,
            label: row.label,
            email: row.email,
            sort_order: row.sort_order,
            remark: row.remark,
            created_at: to_local_datetime(Some(row.created_at)),
            updated_at: to_local_datetime(Some(row.updated_at)),
        }
    }
}

#[derive(Serialize)]
pub struct ClassListResp {
    pub list: Vec<ClassInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
