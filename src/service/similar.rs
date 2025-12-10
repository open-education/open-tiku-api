use crate::constant::meta;
use crate::util::{file, string};
use log::error;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

pub fn read_question_similar_index(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
) -> Result<HashMap<String, Vec<String>>, Error> {
    let index_path = format!(
        "{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(textbook_key),
        string::underline_to_slash(catalog_key),
        meta::QUESTION_SIMILAR_INDEX_NAME
    );
    // 读取索引文件时如果文件不存在则返回空, 写入时才主动创建文件
    let index_content =
        file::read_small_file(index_path, true).unwrap_or_else(|_| "{}".to_string());
    let index_list: HashMap<String, Vec<String>> = serde_json::from_str(&index_content)?;
    Ok(index_list)
}

pub fn write_similar_index(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
    similar_index_map: &HashMap<String, Vec<String>>,
) -> Result<bool, Error> {
    // write question similar index file
    let index_path = format!(
        "{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&textbook_key),
        string::underline_to_slash(&catalog_key),
        meta::QUESTION_SIMILAR_INDEX_NAME
    );

    match serde_json::to_string(&similar_index_map) {
        Ok(content) => match file::write_small_file(&index_path, &content) {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Failed to save question similar index to json file: {}", e);
                Err(Error::new(ErrorKind::Other, "变式题索引维护保存错误"))?
            }
        },
        Err(e) => {
            error!(
                "Failed to serialize question similar index to json str: {}",
                e
            );
            Err(Error::new(ErrorKind::Other, "变式题索引维护序列化错误"))?
        }
    }
}
