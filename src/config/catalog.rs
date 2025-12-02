use crate::constant::meta;
use crate::util::{file, string};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Catalog {
    pub label: String,
    pub key: String,
    pub order: i8,
    pub children: Option<Vec<Catalog>>,
}

// key: pip_chinese_senior_1
pub fn get_catalogs(key: &str) -> Result<Vec<Catalog>, Error> {
    let key_path: String = format!(
        "{}/{}/{}",
        meta::META_PATH,
        string::underline_to_slash(key),
        meta::CATALOG_NAME
    );
    let contents = file::read_small_file(key_path, true)?;
    let catalogs: Vec<Catalog> = serde_json::from_str(&contents)?;
    Ok(catalogs)
}
