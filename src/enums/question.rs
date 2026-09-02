// 题目列表请求来源
#[derive(PartialEq, Eq)]
pub enum QuestionPageSource {
    List,       // 普通题库列表请求
    MyQuestion, // 我的题目
    MyReview,   // 我的审核
}

impl QuestionPageSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "list" => Some(QuestionPageSource::List),
            "myQuestion" => Some(QuestionPageSource::MyQuestion),
            "myReview" => Some(QuestionPageSource::MyReview),
            _ => None,
        }
    }
}

// 审核状态枚举
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuestionStatus {
    Draft = 0,     // 0: 草稿
    Pending = 1,   // 1: 待审核
    Published = 2, // 2: 已发布
    Rejected = 3,  // 3: 被拒绝
}

impl QuestionStatus {
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuestionRelationType {
    Similar = 1,  // 变式题
    Original = 2, // 课本原题
    Base = 3,     // 母题
}

impl QuestionRelationType {
    pub fn from_i16(val: i16) -> Option<QuestionRelationType> {
        match val {
            1 => Some(QuestionRelationType::Similar),
            2 => Some(QuestionRelationType::Original),
            3 => Some(QuestionRelationType::Base),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}
