// 用户角色
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoleType {
    Normal = 1,  // 1 普通
    Student = 2, // 2 学生, 单独的 class_student 表
    Teacher = 3, // 3 教师
}

impl RoleType {
    pub fn desc(value: i16) -> String {
        match value {
            1 => "普通用户".to_string(),
            2 => "学生账户".to_string(),
            3 => "教师账户".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// 登录平台类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    Github = 1,
    QQ = 2,
}

impl ProviderType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Github),
            2 => Some(Self::QQ),
            _ => None, // 未知平台登录无法处理
        }
    }

    pub fn desc(value: i16) -> String {
        match value {
            1 => "GitHub".to_string(),
            2 => "QQ".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// 用户状态
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Active = 1,     // 1 正常
    Paused = 2,     // 2 暂停
    Forbidden = 20, // 20 封禁
}

impl StatusType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Active),
            2 => Some(Self::Paused),
            3 => Some(Self::Forbidden),
            _ => None,
        }
    }

    pub fn desc(value: i16) -> String {
        match value {
            1 => "激活".to_string(),
            2 => "暂停".to_string(),
            20 => "封禁".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// 用户来源
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UserSource {
    User = 1,    // 普通第三方用户
    Student = 2, // 学生账户
}

impl UserSource {
    pub fn desc(value: i16) -> String {
        match value {
            1 => "普通第三方用户".to_string(),
            2 => "学生账户".to_string(),
            _ => "未知".to_string(),
        }
    }
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::User),
            2 => Some(Self::Student),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}
