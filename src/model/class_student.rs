use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, QueryBuilder};

#[derive(Clone, FromRow)]
pub struct ClassStudent {
    pub id: i64,
    pub class_id: i64,
    pub user_id: i64,
    pub account: String,
    pub password: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    pub remark: String,
    pub last_login_time: Option<DateTime<Utc>>,
    pub login_count: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ClassStudent {
    pub async fn batch_insert(pool: &PgPool, records: &[Self]) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb = QueryBuilder::new(
            "INSERT INTO class_student (class_id, user_id, account, password, status, remark, last_login_time, login_count) ",
        );
        qb.push_values(records, |mut b, r| {
            b.push_bind(r.class_id)
                .push_bind(r.user_id)
                .push_bind(&r.account)
                .push_bind(&r.password)
                .push_bind(r.status)
                .push_bind(&r.remark)
                .push_bind(r.last_login_time)
                .push_bind(r.login_count);
        });
        Ok(qb.build().execute(pool).await?.rows_affected())
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, class_id, user_id, account, password, status, remark, 
         last_login_time, login_count, created_at, updated_at
         FROM class_student
         WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_ids(pool: &PgPool, ids: Vec<i64>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"SELECT id, class_id, user_id, account, password, status, remark,
         last_login_time, login_count, created_at, updated_at
         FROM class_student
         WHERE id = ANY($1)"#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    pub async fn update_by_id(pool: &PgPool, req: &Self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query::<_>(
            r#"
            UPDATE class_student
            SET
                account = $1,
                password = $2,
                status = $3,
                remark = $4,
                last_login_time = $5,
                login_count = $6,
                updated_at = NOW()
            WHERE id = $7
            "#,
        )
        .bind(req.account.clone())
        .bind(req.password.clone())
        .bind(req.status)
        .bind(req.remark.clone())
        .bind(req.last_login_time)
        .bind(req.login_count)
        .bind(req.id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_by_class_ids(
        pool: &PgPool,
        class_ids: Vec<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT
            id, class_id, user_id, account, password,
            status, remark, last_login_time, login_count,
            created_at, updated_at
        FROM class_student
        WHERE class_id = ANY($1)
        ORDER BY id DESC
        "#,
        )
        .bind(class_ids)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_account(
        pool: &PgPool,
        account: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT
            id, class_id, user_id, account, password,
            status, remark, last_login_time, login_count,
            created_at, updated_at
        FROM class_student
        WHERE account = $1
        "#,
        )
        .bind(account)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user_id(pool: &PgPool, user_id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT
            id, class_id, user_id, account, password,
            status, remark, last_login_time, login_count,
            created_at, updated_at
        FROM class_student
        WHERE user_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user_ids(
        pool: &PgPool,
        user_ids: Vec<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT
            id, class_id, user_id, account, password,
            status, remark, last_login_time, login_count,
            created_at, updated_at
        FROM class_student
        WHERE user_id = ANY($1)
        "#,
        )
        .bind(user_ids)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_accounts(
        pool: &PgPool,
        accounts: &[String],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, class_id, user_id, account, password, status, remark, created_at, updated_at
            FROM class_student
            WHERE account = ANY($1)
            "#,
        )
        .bind(accounts)
        .fetch_all(pool)
        .await
    }

    pub async fn delete_by_class_id(pool: &PgPool, class_id: i64) -> Result<u64, sqlx::Error> {
        let row = sqlx::query("DELETE FROM class_student WHERE class_id = $1")
            .bind(class_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(row)
    }
}
