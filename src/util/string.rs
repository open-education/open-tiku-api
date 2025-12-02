use std::io::{Error, ErrorKind};

/// 将字符串下划线替换为文件目录分隔符斜线
/// pep_chinese_senior_1 -> pep/chinese/senior/1
pub fn underline_to_slash(input: &str) -> String {
    input.replace("_", "/")
}

pub fn get_first_part(input: &str) -> Result<&str, Error> {
    match input.splitn(2, '_').next() {
        Some(part) => Ok(part),
        None => Err(Error::new(ErrorKind::Other, "Empty string")),
    }
}

/// 将字符串按指定的分隔符分割，并保留制定的数量，最终按指定的连接符连接
/// pep_chinese_senior_1
/// 保留前三个并用斜线连接 take_first_n_parts(key, '_', '/', 3)
/// pep/chinese/senior
/// 字符串不适合分割会返回错误，而非返回整体的字符串
pub fn take_first_n_parts(
    input: &str,
    delimiter: char,
    join: char,
    n: usize,
) -> Result<String, Error> {
    let parts: Vec<&str> = input.split(delimiter).collect();
    if parts.len() < n {
        Err(Error::new(
            ErrorKind::Other,
            "The string is not suitable for splitting",
        ))?
    }
    Ok(parts[..n].join(&join.to_string()))
}

pub fn has_content(input: &str) -> bool {
    !input.trim().is_empty()
}
