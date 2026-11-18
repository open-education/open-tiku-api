use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

// 个性化定制

#[derive(FromRow)]
pub struct Custom {
    pub id: Option<i64>,
    pub custom_type: i16,
    pub textbook: Json<CustomTextbook>,
}

// 定制菜单
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomTextbook {
    two_level_id: Option<i32>,            // 是否选择第2层菜单
    five_level_id: Option<i32>,           // 是否选择了第5层菜单, 用户展开题型列表
    five_level_keys: Option<Vec<String>>, // 前5层菜单 key
}

impl Custom {
    pub async fn save(
        pool: &PgPool,
        id: Option<i64>,
        author_id: i64,
        custom_type: i16,
        textbook: CustomTextbook,
    ) -> Result<i64, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            r"
        INSERT INTO custom (id, author_id, custom_type, textbook)
        VALUES (
            COALESCE($1, nextval('custom_id_seq')),
            $2, $3, $4
        )
        ON CONFLICT (id) DO UPDATE SET
            author_id = EXCLUDED.author_id,
            custom_type = EXCLUDED.custom_type,
            textbook = EXCLUDED.textbook
        RETURNING id
        ",
        )
        .bind(id)
        .bind(author_id)
        .bind(custom_type)
        .bind(Json(textbook))
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    pub async fn list(pool: &PgPool, author_id: i64) -> sqlx::Result<Vec<Self>> {
        let rows = sqlx::query_as::<_, Self>(
            r"
            SELECT id, author_id, custom_type, textbook
            FROM custom
            WHERE author_id = $1
            ",
        )
        .bind(author_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
