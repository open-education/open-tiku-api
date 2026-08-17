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
