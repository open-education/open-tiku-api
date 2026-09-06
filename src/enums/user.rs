// 用户角色
#[repr(i16)]
pub enum RoleType {
    Normal = 1,  // 1 普通
    Student = 2, // 2 学生, 单独的 class_student 表
    Teacher = 3, // 3 教师
}

impl RoleType {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "普通用户",
            2 => "学生账户",
            3 => "教师账户",
            _ => "Unknown",
        }
    }
}

// 登录平台类型
#[repr(i16)]
pub enum ProviderType {
    Github = 1,
    QQ = 2,
}

impl ProviderType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Github),
            2 => Some(Self::QQ),
            _ => None, // 未知平台登录无法处理
        }
    }

    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "GitHub",
            2 => "QQ",
            _ => "Unknown",
        }
    }
}

// 用户状态
#[repr(i16)]
pub enum StatusType {
    Active = 1,     // 1 正常
    Paused = 2,     // 2 暂停
    Forbidden = 20, // 20 封禁
}

impl StatusType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Active),
            2 => Some(Self::Paused),
            3 => Some(Self::Forbidden),
            _ => None,
        }
    }

    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "激活",
            2 => "暂停",
            20 => "封禁",
            _ => "Unknown",
        }
    }
}

// 用户来源
#[repr(i16)]
pub enum UserSource {
    User = 1,    // 普通第三方用户
    Student = 2, // 学生账户
}

impl UserSource {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "普通第三方用户",
            2 => "学生账户",
            _ => "未知",
        }
    }
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::User),
            2 => Some(Self::Student),
            _ => None,
        }
    }
}
