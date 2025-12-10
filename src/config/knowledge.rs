use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Knowledge {
    pub label: String,
    pub key: String,
    pub order: i8,
}

pub fn get_knowledge(meta_path: &str, key: &str) -> Result<Vec<Knowledge>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta_path,
        string::underline_to_slash(key),
        meta::KNOWLEDGE_NAME
    );
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let knowledge: Vec<Knowledge> = serde_json::from_str(&contents)?;
    Ok(knowledge)
}

#[derive(Serialize, Deserialize)]
pub struct KnowledgeInfo {
    pub label: String,
    pub key: String,
    pub order: i8,
    pub children: Option<Vec<KnowledgeInfo>>,
}

// key: math_pep_junior_x
pub fn get_knowledge_info(meta_path: &str, key: &str) -> Result<Vec<KnowledgeInfo>, Error> {
    let key_path: String = format!(
        "{}/{}/{}/{}.json",
        meta_path,
        string::take_first_n_parts(key, '_', '/', 3)?,
        meta::KNOWLEDGE_PATH,
        string::get_last_part(&key)
    );
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let knowledge_info: Vec<KnowledgeInfo> = serde_json::from_str(&contents)?;
    Ok(knowledge_info)
}
