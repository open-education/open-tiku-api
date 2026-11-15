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
