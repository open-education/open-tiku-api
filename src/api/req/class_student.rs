use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStudentReq {
    pub class_id: i64,
    // 是否增量导入
    pub incremental: bool,
    // 账户名称是英文逗号分割的字符串
    pub accounts: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStudentListReq {
    pub class_ids: Vec<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStudentEditReq {
    pub id: i64,
    pub class_id: i64,
    pub account: String,
    pub status: i16,
    pub reset_pwd: bool,
    pub remark: String,
}
