use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct City {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub pinyin_initial: String,
    pub level: i16,
}

impl City {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(r"SELECT * FROM city")
            .fetch_all(pool)
            .await
    }
}
