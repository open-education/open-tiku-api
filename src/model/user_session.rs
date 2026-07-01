use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

// 用户登录管理

#[derive(FromRow)]
pub struct UserSession {
    pub id: Option<i64>,
    pub user_id: i64,
    pub source: i16,
    pub token_type: i16,
    pub token: String,
    pub expired_at: DateTime<Utc>,
    pub renew_cnt: i16,
    pub use_cnt: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 用户来源
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Third = 1,   // 1 third 第三方账户登录
    Student = 2, // 2 student 学生账号
}

impl SourceType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Third),
            2 => Some(Self::Student),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// token 类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Temp = 1,  // 1 是临时 token 用于换取真实登录 token, 不可以续期过期删除
    Login = 2, // 2 登录 token，每次访问需要续期
}

impl TokenType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Temp),
            2 => Some(Self::Login),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

impl UserSession {
    pub async fn save(pool: &PgPool, session: Self) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        INSERT INTO user_session (
            id, user_id, source, token_type, token, expired_at, renew_cnt, use_cnt
        )
        VALUES (
            COALESCE($1, nextval('user_session_id_seq')),
            $2, $3, $4, $5, $6, $7, $8
        )
        ON CONFLICT (id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            source = EXCLUDED.source,
            token_type = EXCLUDED.token_type,
            token = EXCLUDED.token,
            expired_at = EXCLUDED.expired_at,
            renew_cnt = EXCLUDED.renew_cnt,
            use_cnt = EXCLUDED.use_cnt,
            updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#,
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(session.source)
        .bind(session.token_type)
        .bind(&session.token)
        .bind(session.expired_at)
        .bind(session.renew_cnt)
        .bind(session.use_cnt)
        .fetch_one(pool)
        .await
    }

    pub async fn find_one_by_user(
        pool: &PgPool,
        user_id: i64,
        source: i16,
        token_type: i16,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_session
        WHERE user_id = $1 AND source = $2 AND token_type = $3
        "#,
        )
        .bind(user_id)
        .bind(source)
        .bind(token_type)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_one_by_token(
        pool: &PgPool,
        token: &str,
        token_type: i16,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_session
        WHERE token = $1 AND token_type = $2
        "#,
        )
        .bind(token)
        .bind(token_type)
        .fetch_optional(pool)
        .await
    }
}
