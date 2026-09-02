use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Serialize, Deserialize, Type, PartialEq, Clone)]
#[repr(i16)]
pub enum StudentStatus {
    Active = 1,   // 激活
    Pause = 2,    // 暂停
    Disabled = 3, // 停用
}

impl StudentStatus {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "激活".to_string(),
            2 => "暂停".to_string(),
            _ => "停用".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Self {
        match value {
            1 => Self::Active,
            2 => Self::Pause,
            _ => Self::Disabled,
        }
    }

    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}
