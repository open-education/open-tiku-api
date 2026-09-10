pub enum TypeCode {
    Type,       // 类型
    Tag,        // 标签
    Dimension,  // 核心素养
    Level,      // 分层体系
    Scene,      // 适用场景
    MistakeTip, // 常见错误
}

impl TypeCode {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "question_type" => Some(TypeCode::Type),
            "question_tag" => Some(TypeCode::Tag),
            "question_dimension" => Some(TypeCode::Dimension),
            "question_level" => Some(TypeCode::Level),
            "question_scene" => Some(TypeCode::Scene),
            "question_mistake_tip" => Some(TypeCode::MistakeTip),
            _ => None,
        }
    }
}
