/// 图片存储文件路径
pub const IMAGE_NAME: &str = "images";
/// 最大文件大小（字节）
pub const MAX_IMAGE_SIZE: usize = 1 * 1024 * 1024;
/// 允许的文件扩展名
pub const ALLOW_IMAGE_EXTENSION: [&str; 4] = ["jpg", "jpeg", "png", "gif"];
/// 图片名称存储长度
pub const IMAGE_NAME_LEN: usize = 10;
/// 图片访问 api 前缀, 由 nginx 决定
pub const IMAGE_READ_PREFIX: &str = "api";
