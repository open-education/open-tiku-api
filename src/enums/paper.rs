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

#[repr(i16)]
pub enum PaperStatus {
    Draft = 1,     // 1: 草稿
    Pending = 2,   // 2: 待审核
    Published = 3, // 3: 已发布
    Homework = 4,  // 4. 已布置作业
    Rejected = 10, // 10: 被拒绝
}

impl PaperStatus {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "草稿",
            2 => "待审核",
            3 => "已发布",
            4 => "已布置作业",
            10 => "被拒绝",
            _ => "未知状态",
        }
    }

    pub fn from_i16(code: i16) -> Self {
        match code {
            1 => Self::Draft,
            2 => Self::Pending,
            3 => Self::Published,
            4 => Self::Homework,
            10 => Self::Rejected,
            _ => Self::Draft,
        }
    }
}

// 试卷类型
#[repr(i16)]
pub enum PaperType {
    Top,
    Gen,
}
