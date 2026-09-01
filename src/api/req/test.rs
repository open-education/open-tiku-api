use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListReq {
    #[serde(rename(deserialize = "startDate"))]
    pub start_date: String,
    #[serde(rename(deserialize = "endDate"))]
    pub end_date: String,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}
