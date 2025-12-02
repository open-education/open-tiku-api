use crate::config::stage::Stage;
use crate::constant::meta;
use crate::util::file;
use log::error;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

#[derive(Serialize, Deserialize)]
pub struct Publisher {
    pub label: String,
    pub key: String,
    pub order: i8,
    pub children: Vec<Subject>,
}

#[derive(Serialize, Deserialize)]
pub struct Subject {
    pub label: String,
    pub key: String,
    pub order: i8,
    #[serde(skip_deserializing)]
    pub children: Vec<Stage>,
}

pub fn get_publishers() -> Result<Vec<Publisher>, Error> {
    let key_path: String = format!("{}/{}", meta::META_PATH, meta::PUBLISHER_NAME);
    let contents = file::read_small_file(key_path, true)?;
    let publishers: Vec<Publisher> = serde_json::from_str(&contents)?;
    Ok(publishers)
}

pub fn get_publisher_by_key(key: &str) -> Result<Publisher, Error> {
    let publishers = get_publishers()?;
    match publishers.into_iter().find(|item| item.key == key) {
        Some(item) => Ok(item),
        None => {
            error!("No such publisher label: {}", key);
            Err(Error::new(
                ErrorKind::NotFound,
                format!("No such publisher label: {}", key),
            ))
        }
    }
}
