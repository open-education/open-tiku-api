use crate::config::knowledge::Knowledge;
use crate::config::textbook::Textbook;
use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Stage {
    pub label: String,
    pub key: String,
    pub order: i8,

    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "textbookList"))]
    pub textbook_list: Vec<Textbook>,
    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "knowledgeList"))]
    pub knowledge_list: Vec<Knowledge>,
}

pub fn get_stages(meta_path: &str, key: &str) -> Result<Vec<Stage>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta_path,
        string::underline_to_slash(key),
        meta::STAGE_NAME
    );
    let contents = file::read_small_file(key_path, false).unwrap_or("[]".to_string());
    let stages: Vec<Stage> = serde_json::from_str(&contents)?;
    Ok(stages)
}
