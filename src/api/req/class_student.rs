use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClassStudentReq {
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    // 是否增量导入
    pub incremental: bool,
    // 账户名称是英文逗号分割的字符串
    pub accounts: String,
}

#[derive(Deserialize)]
pub struct ClassStudentListReq {
    #[serde(rename(deserialize = "classIds"))]
    pub class_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct ClassStudentEditReq {
    pub id: i64,
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    pub account: String,
    pub status: i16,
    #[serde(rename(deserialize = "resetPwd"))]
    pub reset_pwd: bool,
    pub remark: String,
}
