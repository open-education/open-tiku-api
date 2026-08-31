use crate::api::req::user::UserEditReq;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

// 第三方用户登录信息

#[derive(FromRow)]
pub struct UserIdentity {
    pub id: Option<i64>,
    pub user_id: i64,
    pub provider: i16,
    pub provider_user_id: String,
    pub provider_username: Option<String>,
    pub provider_email: Option<String>,
    pub last_login_time: Option<DateTime<Utc>>,
    pub login_count: i64,
    pub role: i16,
    pub status: i16,
    pub remark: String,
    // 创建更新时间
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// 登录平台类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    Github = 1,
    QQ = 2,
}

impl ProviderType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Github),
            2 => Some(Self::QQ),
            _ => None, // 未知平台登录无法处理
        }
    }

    pub fn desc(value: i16) -> String {
        match value {
            1 => "GitHub".to_string(),
            2 => "QQ".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// 用户状态
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Active = 1,     // 1 正常
    Paused = 2,     // 2 暂停
    Forbidden = 20, // 20 封禁
}

impl StatusType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Active),
            2 => Some(Self::Paused),
            3 => Some(Self::Forbidden),
            _ => None,
        }
    }

    pub fn desc(value: i16) -> String {
        match value {
            1 => "激活".to_string(),
            2 => "暂停".to_string(),
            20 => "封禁".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

impl UserIdentity {
    pub async fn save(pool: &PgPool, identity: &Self) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        INSERT INTO user_identity (
            id, user_id, provider, provider_user_id, provider_username,
            provider_email, last_login_time, login_count, role, status, remark
        )
        VALUES (
            COALESCE($1, nextval('user_identity_id_seq')),
            $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
        )
        ON CONFLICT (id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            provider = EXCLUDED.provider,
            provider_user_id = EXCLUDED.provider_user_id,
            provider_username = EXCLUDED.provider_username,
            provider_email = EXCLUDED.provider_email,
            last_login_time = EXCLUDED.last_login_time,
            login_count = EXCLUDED.login_count,
            role = EXCLUDED.role,
            status = EXCLUDED.status,
            remark = EXCLUDED.remark,
            updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#,
        )
        .bind(identity.id)
        .bind(identity.user_id)
        .bind(identity.provider)
        .bind(&identity.provider_user_id)
        .bind(&identity.provider_username)
        .bind(&identity.provider_email)
        .bind(identity.last_login_time)
        .bind(identity.login_count)
        .bind(identity.role)
        .bind(identity.status)
        .bind(identity.remark.clone())
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_provider(
        pool: &PgPool,
        provider: i16,
        provider_user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_identity
        WHERE provider = $1 AND provider_user_id = $2
        "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user_id(pool: &PgPool, user_id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_identity
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
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM user_identity
        WHERE user_id = ANY($1)
        "#,
        )
        .bind(user_ids)
        .fetch_all(pool)
        .await
    }

    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM user_identity"#)
            .fetch_one(pool)
            .await
    }

    pub async fn list(pool: &PgPool, limit: i32, offset: i32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT *
        FROM user_identity
        ORDER BY id DESC
        LIMIT $1 OFFSET $2
        "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn update_by_id(pool: &PgPool, req: UserEditReq) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
        UPDATE user_identity
        SET status = $2,
            remark = $3,
            updated_at = $4
        WHERE id = $1
        "#,
        )
        .bind(req.id)
        .bind(req.status)
        .bind(req.remark)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
