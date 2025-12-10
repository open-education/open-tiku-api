use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct QuestionType {
    label: String,
    key: String,
    order: i8,
}

pub fn get_question_types(meta_path: &str, key: &str) -> Result<Vec<QuestionType>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta_path,
        string::take_first_n_parts(key, '_', '/', 3)?,
        meta::QUESTION_TYPE_NAME
    );
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let questions: Vec<QuestionType> = serde_json::from_str(&contents)?;
    Ok(questions)
}
