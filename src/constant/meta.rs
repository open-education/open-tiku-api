/// 一级目录学科文件名
pub const SUBJECT_NAME: &str = "subject.json";
/// 二级目录出版社文件名
pub const PUBLISHER_NAME: &str = "publisher.json";
/// 三级目录学段文件名
pub const STAGE_NAME: &str = "stage.json";
/// 图书文件名
pub const TEXTBOOK_NAME: &str = "textbook.json";
/// 知识点文件名
pub const KNOWLEDGE_NAME: &str = "knowledge.json";
/// 知识点文件路径
pub const KNOWLEDGE_PATH: &str = "knowledge";
/// 图书目录文件名
pub const CATALOG_NAME: &str = "catalog.json";
/// 问题类型文件名
pub const QUESTION_TYPE_NAME: &str = "question_type.json";
/// 问题标签文件名
pub const TAG_NAME: &str = "tag.json";
/// 图片存储文件路径
pub const IMAGE_NAME: &str = "images";
/// 最大文件大小（字节）
pub const MAX_IMAGE_SIZE: usize = 1 * 1024 * 1024;
/// 允许的文件扩展名
pub const ALLOW_IMAGE_EXTENSION: [&str; 4] = ["jpg", "jpeg", "png", "gif"];
/// 图片名称存储长度
pub const IMAGE_NAME_LEN: usize = 10;
/// 题目索引文件名
pub const QUESTION_INDEX_NAME: &str = "index.json";
// 索引存储长度
pub const QUESTION_INDEX_LENGTH: usize = 5;
/// 标题
pub const QUESTION_TITLE_NAME: &str = "title.md";
/// 补充
pub const QUESTION_MENTION_NAME: &str = "mention.md";
/// 选项A
pub const QUESTION_A_NAME: &str = "a.md";
/// 选项B
pub const QUESTION_B_NAME: &str = "b.md";
/// 选项C
pub const QUESTION_C_NAME: &str = "c.md";
///选项D
pub const QUESTION_D_NAME: &str = "d.md";
/// 选项E
pub const QUESTION_E_NAME: &str = "e.md";
/// 答案
pub const QUESTION_ANSWER_NAME: &str = "answer.md";
/// 知识点
pub const QUESTION_KNOWLEDGE_NAME: &str = "knowledge.md";
/// 解题分析
pub const QUESTION_ANALYZE_NAME: &str = "analyze.md";
/// 解题过程
pub const QUESTION_PROCESS_NAME: &str = "process.md";
/// 备注
pub const QUESTION_REMARK_NAME: &str = "remark.md";
/// 变氏题索引关联文件名
pub const QUESTION_SIMILAR_INDEX_NAME: &str = "similar.json";
