use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, QueryBuilder, Type};

#[derive(Serialize, Deserialize, Type, PartialEq, Clone)]
#[repr(i16)]
pub enum StudentStatus {
    Active = 1,   // 激活
    Pause = 2,    // 暂停
    Disabled = 3, // 停用
}

impl StudentStatus {
    pub fn desc(code: i16) -> String {
        match code {
            1 => "激活".to_string(),
            2 => "暂停".to_string(),
            _ => "停用".to_string(),
        }
    }

    pub fn from_i16(value: i16) -> Self {
        match value {
            1 => Self::Active,
            2 => Self::Pause,
            _ => Self::Disabled,
        }
    }

    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}

#[derive(Clone, FromRow)]
pub struct ClassStudent {
    pub id: i64,
    pub class_id: i64,
    pub account: String,
    pub password: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    pub remark: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ClassStudent {
    pub async fn batch_insert(pool: &PgPool, records: &[Self]) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb = QueryBuilder::new(
            "INSERT INTO class_student (class_id, account, password, status, remark) ",
        );
        qb.push_values(records, |mut b, r| {
            b.push_bind(r.class_id)
                .push_bind(&r.account)
                .push_bind(&r.password)
                .push_bind(r.status)
                .push_bind(&r.remark);
        });
        Ok(qb.build().execute(pool).await?.rows_affected())
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id, class_id, account, password, status, remark, created_at, updated_at
            FROM class_student
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update_by_id(pool: &PgPool, req: &Self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE class_student
            SET
                class_id = $1,
                account = $2,
                password = $3,
                status = $4,
                remark = $5,
                updated_at = NOW()
            WHERE id = $6
            "#,
            req.class_id,
            req.account,
            req.password,
            req.status,
            req.remark,
            req.id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_by_class_id(pool: &PgPool, class_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id, class_id, account, password, status, remark, created_at, updated_at
            FROM class_student
            WHERE class_id = $1
            ORDER BY id DESC
            "#,
            class_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_account(
        pool: &PgPool,
        account: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id, class_id, account, password, status, remark, created_at, updated_at
            FROM class_student
            WHERE account = $1
            "#,
            account
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_accounts(
        pool: &PgPool,
        accounts: &[String],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id, class_id, account, password, status, remark, created_at, updated_at
            FROM class_student
            WHERE account = ANY($1)
            "#,
            accounts
        )
        .fetch_all(pool)
        .await
    }

    pub async fn delete_by_id(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
        let row = sqlx::query!("DELETE FROM class_student WHERE id = $1", id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(row)
    }

    pub async fn delete_by_class_id(pool: &PgPool, class_id: i64) -> Result<u64, sqlx::Error> {
        let row = sqlx::query!("DELETE FROM class_student WHERE class_id = $1", class_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(row)
    }
}
