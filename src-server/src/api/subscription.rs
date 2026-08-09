//! Subscription API — server 为订阅权威源

use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;

use crate::{auth::jwt::AuthClaims, config::CONFIG};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_my_subscription).service(dev_upgrade);
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionResponse {
    pub user_id: String,
    pub tier: String,
    pub status: String,
    pub expires_at: Option<String>,
}

struct SubscriptionRow {
    tier: String,
    status: String,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn get_or_create(pool: &PgPool, user_id: uuid::Uuid) -> Result<SubscriptionRow, sqlx::Error> {
    let existing = sqlx::query!(
        "SELECT tier, status, expires_at FROM subscriptions WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        // 非 free 且已过期：懒降级为 free 并写回库
        let expired = row.tier != "free"
            && row
                .expires_at
                .map(|t| t < chrono::Utc::now())
                .unwrap_or(false);
        if expired {
            let row = sqlx::query!(
                "UPDATE subscriptions SET tier = 'free', expires_at = NULL, updated_at = NOW()
                 WHERE user_id = $1
                 RETURNING tier, status, expires_at",
                user_id
            )
            .fetch_one(pool)
            .await?;
            return Ok(SubscriptionRow {
                tier: row.tier,
                status: row.status,
                expires_at: row.expires_at,
            });
        }
        return Ok(SubscriptionRow {
            tier: row.tier,
            status: row.status,
            expires_at: row.expires_at,
        });
    }

    // 并发首请求可能同时 SELECT 未命中，INSERT 撞 user_id UNIQUE：
    // ON CONFLICT DO NOTHING 后重新 SELECT，以已存在的一行为准
    sqlx::query!(
        "INSERT INTO subscriptions (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
        user_id
    )
    .execute(pool)
    .await?;

    let row = sqlx::query!(
        "SELECT tier, status, expires_at FROM subscriptions WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(SubscriptionRow {
        tier: row.tier,
        status: row.status,
        expires_at: row.expires_at,
    })
}

pub(crate) async fn upsert_tier(
    pool: &PgPool,
    user_id: uuid::Uuid,
    tier: &str,
    pro_days: Option<i64>,
) -> Result<SubscriptionRow, sqlx::Error> {
    let expires_at = if tier == "free" {
        None
    } else if let Some(days) = pro_days {
        Some(chrono::Utc::now() + chrono::Duration::days(days))
    } else if tier == "pro" {
        Some(chrono::Utc::now() + chrono::Duration::days(30))
    } else {
        None
    };
    sqlx::query!(
        "INSERT INTO subscriptions (user_id, tier, expires_at) VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE SET tier = $2, expires_at = $3, updated_at = NOW()",
        user_id,
        tier,
        expires_at
    )
    .execute(pool)
    .await?;
    // 写后读回真实行（Task 4 admin 调订阅复用）
    let row = sqlx::query!(
        "SELECT tier, status, expires_at FROM subscriptions WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(SubscriptionRow {
        tier: row.tier,
        status: row.status,
        expires_at: row.expires_at,
    })
}

fn to_response(user_id: uuid::Uuid, row: SubscriptionRow) -> SubscriptionResponse {
    SubscriptionResponse {
        user_id: user_id.to_string(),
        tier: row.tier,
        status: row.status,
        expires_at: row.expires_at.map(|t| t.to_rfc3339()),
    }
}

#[get("/subscription/me")]
async fn get_my_subscription(claims: AuthClaims, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match claims.sub.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid user ID"})),
    };
    match get_or_create(pool.get_ref(), user_id).await {
        Ok(row) => HttpResponse::Ok().json(to_response(user_id, row)),
        Err(e) => {
            log::error!("subscription query failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct DevUpgradeRequest {
    tier: String,
}

#[post("/subscription/dev-upgrade")]
async fn dev_upgrade(
    claims: AuthClaims,
    pool: web::Data<PgPool>,
    body: web::Json<DevUpgradeRequest>,
) -> impl Responder {
    if !CONFIG.dev_upgrade_enabled {
        return HttpResponse::Forbidden().json(json!({"error": "dev upgrade disabled"}));
    }
    let tier = body.tier.as_str();
    if !["free", "pro", "enterprise"].contains(&tier) {
        return HttpResponse::BadRequest().json(json!({"error": "invalid tier"}));
    }
    let user_id = match claims.sub.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid user ID"})),
    };
    match upsert_tier(pool.get_ref(), user_id, tier, None).await {
        Ok(row) => HttpResponse::Ok().json(to_response(user_id, row)),
        Err(e) => {
            log::error!("dev upgrade failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn me_creates_free_subscription_for_new_user(pool: PgPool) {
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            user_id,
            "t@t.com"
        )
        .execute(&pool)
        .await
        .unwrap();

        let status = get_or_create(&pool, user_id).await.unwrap();
        assert_eq!(status.tier, "free");
        assert_eq!(status.status, "active");
        assert_eq!(status.expires_at, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn dev_upgrade_sets_pro_with_expiry(pool: PgPool) {
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            user_id,
            "t@t.com"
        )
        .execute(&pool)
        .await
        .unwrap();

        let status = upsert_tier(&pool, user_id, "pro", None).await.unwrap();
        assert_eq!(status.tier, "pro");
        assert!(status.expires_at.is_some());

        // 再查回来（跨"设备"读取同一份）
        let again = get_or_create(&pool, user_id).await.unwrap();
        assert_eq!(again.tier, "pro");

        // 降级回 free
        let down = upsert_tier(&pool, user_id, "free", None).await.unwrap();
        assert_eq!(down.tier, "free");
        assert_eq!(down.expires_at, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn expired_pro_is_downgraded_lazily(pool: PgPool) {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            uid,
            "e@t.com"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO subscriptions (user_id, tier, expires_at) VALUES ($1, 'pro', NOW() - INTERVAL '1 day')",
            uid
        )
        .execute(&pool).await.unwrap();

        let row = get_or_create(&pool, uid).await.unwrap();
        assert_eq!(row.tier, "free");
        assert_eq!(row.expires_at, None);
        // 库里也懒更新为 free
        let tier: String =
            sqlx::query_scalar!("SELECT tier FROM subscriptions WHERE user_id = $1", uid)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(tier, "free");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn upsert_tier_with_days_and_reads_back(pool: PgPool) {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            uid,
            "d@t.com"
        )
        .execute(&pool)
        .await
        .unwrap();

        let row = upsert_tier(&pool, uid, "pro", Some(90)).await.unwrap();
        assert_eq!(row.tier, "pro");
        let exp = row.expires_at.unwrap();
        assert!(exp > chrono::Utc::now() + chrono::Duration::days(80));

        // 改回 free 读回真实行
        let row = upsert_tier(&pool, uid, "free", None).await.unwrap();
        assert_eq!(row.tier, "free");
        assert_eq!(row.expires_at, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_or_create_is_idempotent_under_repeated_and_concurrent_calls(pool: PgPool) {
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            user_id,
            "t@t.com"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 同一 user 连续两次：幂等，不报错
        let first = get_or_create(&pool, user_id).await.unwrap();
        let second = get_or_create(&pool, user_id).await.unwrap();
        assert_eq!(first.tier, second.tier);
        assert_eq!(first.status, second.status);

        // 并发调用（模拟并发首请求撞 UNIQUE）：均成功且仍只有一行
        let (a, b) =
            tokio::try_join!(get_or_create(&pool, user_id), get_or_create(&pool, user_id)).unwrap();
        assert_eq!(a.tier, b.tier);

        let count: Option<i64> = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM subscriptions WHERE user_id = $1",
            user_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, Some(1));
    }
}
