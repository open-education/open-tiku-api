use crate::config::textbook::Textbook;
use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Stage {
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub stage_type: String,
    pub order: i8,
    #[serde(skip_deserializing)]
    pub children: Vec<Textbook>,
}

pub fn get_stages(key: &str) -> Result<Vec<Stage>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(key),
        meta::STAGE_NAME
    );
    let contents = file::read_small_file(key_path, true)?;
    let stages: Vec<Stage> = serde_json::from_str(&contents)?;
    Ok(stages)
}
