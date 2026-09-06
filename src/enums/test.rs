#[repr(i16)]
pub enum TestMethod {
    Exercise = 1, // 练习模式
    Exam = 2,     // 考试模式
}

impl TestMethod {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "练习模式",
            2 => "考试模式",
            _ => "未知",
        }
    }

    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Exercise),
            2 => Some(Self::Exam),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
#[repr(i16)]
pub enum TestStatus {
    InProgress = 1, // 进行中
    Done = 2,       // 已交卷
}

impl TestStatus {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "进行中",
            2 => "已交卷",
            _ => "未知",
        }
    }

    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::InProgress),
            2 => Some(Self::Done),
            _ => None,
        }
    }
}

#[repr(i16)]
pub enum TestResult {
    Unanswered = 0, // 未作答
    Correct = 1,    // 正确
    Incorrect = 2,  // 错误
}

impl TestResult {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "正确",
            2 => "错误",
            _ => "未作答",
        }
    }

    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            0 => Some(TestResult::Unanswered),
            1 => Some(Self::Correct),
            2 => Some(Self::Incorrect),
            _ => None,
        }
    }
}
