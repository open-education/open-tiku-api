use crate::constant::meta;
use crate::util::{file, string};
use log::error;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

#[derive(Serialize, Deserialize)]
pub struct QuestionIndex {
    pub id: u64, // 建立题目关联的索引, 后续搜索题目等均读取该文件，然后再根据题目寻找他的内容
    pub left: String, // 对标题进行处理出一个前缀, 题目真实的路径为 id_left, 这样就不关心 id 重复的问题了
    pub question_type: String,
    pub tags: Option<Vec<String>>,
    pub rate_val: Option<String>,
    pub image_names: Option<Vec<String>>,
    pub show_image_val: Option<String>,
    pub show_select_val: Option<String>,
    pub create_time: Option<i64>,
    pub update_time: Option<i64>,
    pub author: Option<String>,
}

pub fn read_question_index(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
) -> Result<Vec<QuestionIndex>, Error> {
    let index_path = format!(
        "{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(textbook_key),
        string::underline_to_slash(catalog_key),
        meta::QUESTION_INDEX_NAME
    );
    // 读取索引文件时如果文件不存在则返回空数组, 写入时才主动创建文件
    let index_content = file::read_small_file(index_path, true).unwrap_or("[]".to_string());
    let index_list: Vec<QuestionIndex> = serde_json::from_str(&index_content)?;
    Ok(index_list)
}

pub fn get_question_index_max_id(index_list: &Vec<QuestionIndex>) -> u64 {
    let mut max_id: u64 = 0;
    for index in index_list {
        if index.id > max_id {
            max_id = index.id;
        }
    }
    max_id
}

pub fn append_write_index(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
    question_index_list: &mut Vec<QuestionIndex>,
    question_index: QuestionIndex,
) -> Result<String, Error> {
    let id = format!("{}_{}", question_index.id, question_index.left);

    question_index_list.push(question_index);

    _ = write_index(meta_path, textbook_key, catalog_key, question_index_list)?;

    Ok(id)
}

pub fn write_index(
    meta_path: &str,
    textbook_key: &str,
    catalog_key: &str,
    question_index_list: &Vec<QuestionIndex>,
) -> Result<bool, Error> {
    // write questio index file
    let index_path = format!(
        "{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&textbook_key),
        string::underline_to_slash(&catalog_key),
        meta::QUESTION_INDEX_NAME
    );

    match serde_json::to_string(&question_index_list) {
        Ok(content) => match file::write_small_file(&index_path, &content) {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Failed to save question index to json file: {}", e);
                Err(Error::new(ErrorKind::Other, "题目上传成功但是索引失败"))?
            }
        },
        Err(e) => {
            error!("Failed to serialize question index to json str: {}", e);
            Err(Error::new(ErrorKind::Other, "题目上传成功但是索引常异"))?
        }
    }
}

pub fn get_question_index(
    id: &str,
    question_index_list: Vec<QuestionIndex>,
) -> Result<QuestionIndex, Error> {
    for index in question_index_list {
        if format!("{}_{}", index.id, index.left) == id {
            return Ok(index);
        }
    }
    Err(Error::new(ErrorKind::Other, "index not found"))
}
