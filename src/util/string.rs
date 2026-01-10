use std::io::{Error, ErrorKind};

/// 将字符串下划线替换为文件目录分隔符斜线
/// pep_chinese_senior_1 -> pep/chinese/senior/1
pub fn underline_to_slash(input: &str) -> String {
    input.replace("_", "/")
}

