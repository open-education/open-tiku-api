use crate::config::publisher::Publisher;
use crate::constant::meta;
use crate::util::file;
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Subject {
    pub label: String,
    pub key: String,
    pub order: i8,
    #[serde(skip_deserializing)]
    pub children: Vec<Publisher>,
}

pub fn get_subjects(meta_path: &str) -> Result<Vec<Subject>, Error> {
    let key_path: String = format!("{}/{}", meta_path, meta::SUBJECT_NAME);
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let subjects: Vec<Subject> = serde_json::from_str(&contents)?;
    Ok(subjects)
}
