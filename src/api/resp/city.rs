use crate::model::city::City;
use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CityResp {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub pinyin_initial: String,
    pub level: i16,
}

impl From<City> for CityResp {
    fn from(raw: City) -> Self {
        Self {
            id: raw.id,
            parent_id: raw.parent_id,
            name: raw.name,
            pinyin_initial: raw.pinyin_initial,
            level: raw.level,
        }
    }
}
