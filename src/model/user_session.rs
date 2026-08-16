use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

// 用户登录管理

// 用户来源
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UserSource {
    User = 1,    // 普通第三方用户
    Student = 2, // 学生账户
}

impl UserSource {
    pub fn from_i16(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::User),
            2 => Some(Self::Student),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        self.clone() as i16
    }
}

#[derive(FromRow)]
pub struct UserSession {
    pub id: Option<i64>,
    pub user_id: i64,
    pub source: i16,
    pub token: String,
    pub expired_at: DateTime<Utc>,
    pub renew_cnt: i16,
    pub client_ip: String,
    pub user_agent: String,
}

impl UserSession {
    pub async fn save(pool: &PgPool, session: Self) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        INSERT INTO user_session (
            id, user_id, source, token, expired_at, renew_cnt, client_ip, user_agent
        )
        VALUES (
            COALESCE($1, nextval('user_session_id_seq')),
            $2, $3, $4, $5, $6, $7, $8
        )
        ON CONFLICT (id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            source = EXCLUDED.source,
            token = EXCLUDED.token,
            expired_at = EXCLUDED.expired_at,
            renew_cnt = EXCLUDED.renew_cnt,
            client_ip = EXCLUDED.client_ip,
            user_agent = EXCLUDED.user_agent,
            updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#,
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(session.source)
        .bind(&session.token)
        .bind(session.expired_at)
        .bind(session.renew_cnt)
        .bind(session.client_ip)
        .bind(session.user_agent)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_session
        WHERE token = $1
        "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await
    }

    // 删除 session
    pub async fn delete_by_id(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_session WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    // 删除过期的 session
    pub async fn delete_expired_sessions(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let result = sqlx::query("DELETE FROM user_session WHERE expired_at <= $1")
            .bind(now)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
