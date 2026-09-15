pub enum PathType {
    Common,
    Chapter,
    Knowledge,
}

impl PathType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "common" => Some(PathType::Common),
            "chapter" => Some(PathType::Chapter),
            "knowledge" => Some(PathType::Knowledge),
            _ => None,
        }
    }

    pub fn desc(self: &Self) -> &'static str {
        match self {
            PathType::Common => "common",
            PathType::Chapter => "chapter",
            PathType::Knowledge => "knowledge",
        }
    }
}
