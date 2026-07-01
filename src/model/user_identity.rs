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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

// 用户角色
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoleType {
    Normal = 1,  // 1 普通
    Student = 2, // 2 学生
    Teacher = 3, // 3 教师
}

impl RoleType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Normal),
            2 => Some(Self::Student),
            3 => Some(Self::Teacher),
            _ => Some(Self::Normal), // 分不清角色就是普通用户
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
            _ => Some(Self::Forbidden), // 分不清状态就是 封禁
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
            provider_email, last_login_time, login_count, role, status
        )
        VALUES (
            COALESCE($1, nextval('user_identity_id_seq')),
            $2, $3, $4, $5, $6, $7, $8, $9, $10
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
}
