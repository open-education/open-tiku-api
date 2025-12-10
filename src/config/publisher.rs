use crate::config::stage::Stage;
use crate::constant::meta;
use crate::util::file;
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Publisher {
    pub label: String,
    pub key: String,
    pub order: i8,
    #[serde(skip_deserializing)]
    pub children: Vec<Stage>,
}

pub fn get_publishers(meta_path: &str, key: &str) -> Result<Vec<Publisher>, Error> {
    let key_path: String = format!("{}/{}/{}", meta_path, key.to_string(), meta::PUBLISHER_NAME);
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let publishers: Vec<Publisher> = serde_json::from_str(&contents)?;
    Ok(publishers)
}
