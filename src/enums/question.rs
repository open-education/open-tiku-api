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
