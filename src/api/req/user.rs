use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeTokenReq {
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
#[serde(rename_all = "camelCase")]
pub struct UserListReq {
    pub page_no: i32,
    pub page_size: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSessionListReq {
    pub page_no: i32,
    pub page_size: i32,
}

#[derive(Deserialize)]
pub struct UserEditReq {
    pub id: i64,
    pub status: i16,
    pub remark: String,
}
