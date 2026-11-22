// 个性化页面来源
pub enum PageSource {
    Default = 1, // 默认, 非登录用户获取系统默认配置, 登录用户没有配置则获取系统默认配置
    My = 2,      // 我的个性化, 只获取我的个性化配置
}

impl PageSource {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Default),
            2 => Some(Self::My),
            _ => None,
        }
    }
}

// 个性化请求来源
pub enum UserSource {
    Person = 1, // 个人
    System = 2, // 系统
}

impl UserSource {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Person),
            2 => Some(Self::System),
            _ => None,
        }
    }
}

// 个性化内容类型
pub enum CustomType {
    Textbook = 1, // 定制教材
}

impl CustomType {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::Textbook),
            _ => None,
        }
    }
}
