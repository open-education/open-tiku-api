#[repr(i16)]
pub enum StudentStatus {
    Active = 1,   // 激活
    Pause = 2,    // 暂停
    Disabled = 3, // 停用
}

impl StudentStatus {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "激活",
            2 => "暂停",
            _ => "停用",
        }
    }

    pub fn from_i16(code: i16) -> Self {
        match code {
            1 => Self::Active,
            2 => Self::Pause,
            _ => Self::Disabled,
        }
    }
}
