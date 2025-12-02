use crate::constant::meta;
use crate::util::file;
use crate::util::string;
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Textbook {
    pub label: String,
    pub key: String,
    pub order: i8,
}

pub fn get_textbooks(subject_name: &str) -> Result<Vec<Textbook>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(subject_name),
        meta::TEXTBOOK_NAME
    );
    let contents = file::read_small_file(key_path, true)?;
    let textbooks: Vec<Textbook> = serde_json::from_str(&contents)?;
    Ok(textbooks)
}
