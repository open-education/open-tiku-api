use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum TaskType {
    UploadQuestion = 1, // 题目上传
}

#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum TaskStatus {
    Waiting = 1, // 待处理
    Running = 2, // 处理中
    Success = 3, // 处理成功
    Failed = 10, // 处理失败
}

impl TaskStatus {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "待处理",
            2 => "处理中",
            3 => "处理成功",
            10 => "处理失败",
            _ => "未知状态",
        }
    }
}
