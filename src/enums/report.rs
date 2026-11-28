// 投诉类型
pub enum ReportType {
    Question = 1, // 题目
    Paper = 2,    // 试卷
}

impl ReportType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Question),
            2 => Some(Self::Paper),
            _ => None,
        }
    }
}
