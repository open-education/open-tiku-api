#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i16)]
pub enum TestMethod {
    Exercise = 1, // 练习模式
    Exam = 2,     // 考试模式
}

impl TestMethod {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "练习模式".to_string(),
            2 => "考试模式".to_string(),
            _ => "未知".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Exercise),
            2 => Some(Self::Exam),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i16)]
pub enum TestStatus {
    InProgress = 1, // 进行中
    Done = 2,       // 已交卷
}

impl TestStatus {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "进行中".to_string(),
            2 => "已交卷".to_string(),
            _ => "未知".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::InProgress),
            2 => Some(Self::Done),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i16)]
pub enum TestResult {
    Unanswered = 0, // 未作答
    Correct = 1,    // 正确
    Incorrect = 2,  // 错误
}

impl TestResult {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "正确".to_string(),
            2 => "错误".to_string(),
            _ => "未作答".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0 => Some(TestResult::Unanswered),
            1 => Some(Self::Correct),
            2 => Some(Self::Incorrect),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}
