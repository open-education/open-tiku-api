pub enum TypeCode {
    QuestionType,       // 类型
    QuestionTag,        // 标签
    QuestionDimension,  // 核心素养
    QuestionLevel,      // 分层体系
    QuestionScene,      // 适用场景
    QuestionMistakeTip, // 常见错误
}

impl TypeCode {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "question_type" => Some(TypeCode::QuestionType),
            "question_tag" => Some(TypeCode::QuestionTag),
            "question_dimension" => Some(TypeCode::QuestionDimension),
            "question_level" => Some(TypeCode::QuestionLevel),
            "question_scene" => Some(TypeCode::QuestionScene),
            "question_mistake_tip" => Some(TypeCode::QuestionMistakeTip),
            _ => None,
        }
    }
}
