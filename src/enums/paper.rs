// 试卷类型
pub enum PaperType {
    Upload = 1, // 上传试卷
    Manual = 2, // 手动组卷
}

impl PaperType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Upload),
            2 => Some(Self::Manual),
            _ => None,
        }
    }
}

// 文件类型列表
pub enum UploadFileType {
    None = 0,     // 无意义
    Pdf = 1,      // PDF 文件
    Word = 2,     // Word 文件
    Image = 3,    // 图片
    Markdown = 4, // Markdown 文档
}

impl UploadFileType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::None),
            1 => Some(Self::Pdf),
            2 => Some(Self::Word),
            3 => Some(Self::Image),
            4 => Some(Self::Markdown),
            _ => None,
        }
    }
}

// 考试类型
pub enum ExamType {
    UnitTest = 1,     // 单元测试
    Midterm = 2,      // 期中考试
    Final = 3,        // 期末考试
    ZhongkaoMock = 4, // 中考模拟
    MonthlyExam = 5,  // 月考
    TopicPaper = 6,   // 专题卷
}

impl ExamType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::UnitTest),
            2 => Some(Self::Midterm),
            3 => Some(Self::Final),
            4 => Some(Self::ZhongkaoMock),
            5 => Some(Self::MonthlyExam),
            6 => Some(Self::TopicPaper),
            _ => None,
        }
    }
}

// 级别类型
pub enum PaperLevelType {
    Standard = 1, // 普通
    Premium = 2,  // 精品
}

impl PaperLevelType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Standard),
            2 => Some(Self::Premium),
            _ => None,
        }
    }
}

// 难度类型
pub enum PaperDifficultyType {
    Easy = 1,   // 容易
    Medium = 2, // 适中
    Hard = 3,   // 困难
}

impl PaperDifficultyType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Easy),
            2 => Some(Self::Medium),
            3 => Some(Self::Hard),
            _ => None,
        }
    }
}

pub enum PaperStatus {
    Draft = 1,     // 1: 草稿
    Pending = 2,   // 2: 待审核
    Published = 3, // 3: 已发布
    Rejected = 10, // 10: 被拒绝
}

impl PaperStatus {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Draft),
            2 => Some(Self::Pending),
            3 => Some(Self::Published),
            10 => Some(Self::Rejected),
            _ => None,
        }
    }
}
