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
        return Ok(SubscriptionRow {
            tier: row.tier,
            status: row.status,
            expires_at: row.expires_at,
        });
    }

    sqlx::query!("INSERT INTO subscriptions (user_id) VALUES ($1)", user_id)
        .execute(pool)
        .await?;
    Ok(SubscriptionRow {
        tier: "free".into(),
        status: "active".into(),
        expires_at: None,
    })
}

async fn upsert_tier(
    pool: &PgPool,
    user_id: uuid::Uuid,
    tier: &str,
) -> Result<SubscriptionRow, sqlx::Error> {
    let expires_at = if tier == "pro" {
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
    Ok(SubscriptionRow {
        tier: tier.into(),
        status: "active".into(),
        expires_at,
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
    match upsert_tier(pool.get_ref(), user_id, tier).await {
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

        let status = upsert_tier(&pool, user_id, "pro").await.unwrap();
        assert_eq!(status.tier, "pro");
        assert!(status.expires_at.is_some());

        // 再查回来（跨"设备"读取同一份）
        let again = get_or_create(&pool, user_id).await.unwrap();
        assert_eq!(again.tier, "pro");

        // 降级回 free
        let down = upsert_tier(&pool, user_id, "free").await.unwrap();
        assert_eq!(down.tier, "free");
        assert_eq!(down.expires_at, None);
    }
}
