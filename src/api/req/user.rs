use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExchangeTokenReq {
    #[serde(rename(deserialize = "tempToken"))]
    pub temp_token: String,
}

#[derive(Deserialize)]
pub struct UserLoginReq {
    // 登录来源 1 第三方用户 2 学生账户
    pub source: i16,

    // 临时 token 登录
    pub token: Option<String>,

    // 用户名密码登录
    pub account: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct UserListReq {
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Deserialize)]
pub struct UserSessionListReq {
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Deserialize)]
pub struct UserEditReq {
    pub id: i64,
    pub status: i16,
    pub remark: String,
}
