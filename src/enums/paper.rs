use serde::{Deserialize, Serialize};
use sqlx::Type;

// 试卷列表请求来源
#[derive(PartialEq, Eq)]
pub enum PaperPageSource {
    List,     // 普通试卷列表请求
    MyPaper,  // 我的试卷
    MyReview, // 我的审核
}

impl PaperPageSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "list" => Some(PaperPageSource::List),
            "myPaper" => Some(PaperPageSource::MyPaper),
            "myReview" => Some(PaperPageSource::MyReview),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Type, PartialEq, Clone)]
#[repr(i16)]
pub enum PaperStatus {
    Draft = 1,     // 1: 草稿
    Pending = 2,   // 2: 待审核
    Published = 3, // 3: 已发布
    Homework = 4,  // 4. 已布置作业
    Rejected = 10, // 10: 被拒绝
}

impl PaperStatus {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "草稿".to_string(),
            2 => "待审核".to_string(),
            3 => "已发布".to_string(),
            4 => "已布置作业".to_string(),
            10 => "被拒绝".to_string(),
            _ => "未知状态".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Self {
        match value {
            1 => Self::Draft,
            2 => Self::Pending,
            3 => Self::Published,
            4 => Self::Homework,
            10 => Self::Rejected,
            _ => Self::Draft,
        }
    }

    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}

// 试卷类型
#[derive(Serialize, Deserialize, Type, PartialEq, Clone)]
#[repr(i16)]
pub enum PaperType {
    Top,
    Gen,
}

impl PaperType {
    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}
