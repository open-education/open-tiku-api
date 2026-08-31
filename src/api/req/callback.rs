use serde::Deserialize;

// 回调参数
#[derive(Deserialize)]
pub struct CallbackQueryReq {
    pub code: Option<String>,
    pub state: Option<String>,
}
