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
