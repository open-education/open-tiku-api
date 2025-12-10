use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Tag {
    label: String,
    key: String,
    order: i8,
}

pub fn get_tags(meta_path: &str, key: &str) -> Result<Vec<Tag>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta_path,
        string::take_first_n_parts(key, '_', '/', 3)?,
        meta::TAG_NAME
    );
    let contents = file::read_small_file(key_path, true).unwrap_or("[]".to_string());
    let tags: Vec<Tag> = serde_json::from_str(&contents)?;
    Ok(tags)
}
