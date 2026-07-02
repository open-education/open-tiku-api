/// 图片存储文件路径
pub const IMAGE_NAME: &str = "images";
/// 其它文件存储路径
pub const FILE_NAME: &str = "files";
/// 最大文件大小（字节）
pub const MAX_IMAGE_SIZE: usize = 1 * 1024 * 1024;
/// 允许的文件扩展名
pub const ALLOW_IMAGE_EXTENSION: [&str; 4] = ["jpg", "jpeg", "png", "gif"];
/// 允许的其它文件扩展名
pub const ALLOW_FILE_EXTENSION: [&str; 1] = ["md"];
/// 图片名称存储长度
pub const IMAGE_NAME_LEN: usize = 10;
/// 临时 token 有效过期分钟数
pub const TEMP_TOKEN_EXPIRED_MINUTE: i64 = 5;
/// 登录 token 有效过期小时数
pub const LOGIN_TOKEN_EXPIRED_HOUR: i64 = 8;
/// 续期 token 有效过期小时数
pub const RENEW_TOKEN_EXPIRED_HOUR: i64 = 4;
