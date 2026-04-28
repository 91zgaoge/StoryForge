//! Auth Handlers — Actix-web HTTP Handlers

use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use super::{
    jwt::{create_token, AuthClaims},
    oauth::{build_oauth_client, exchange_code_and_get_user},
    LoginResponse, OAuthProvider, UserResponse,
};

/// 内存存储：state -> (provider, pkce_verifier)
static OAUTH_STATE_STORE: Lazy<Mutex<HashMap<String, (String, String)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(auth_config)
        .service(oauth_start)
        .service(oauth_callback)
        .service(logout)
        .service(get_me);
}

/// GET /api/auth/config
#[get("/auth/config")]
async fn auth_config() -> impl Responder {
    use crate::config::CONFIG;
    HttpResponse::Ok().json(serde_json::json!({
        "google_enabled": CONFIG.google_client_id.is_some(),
        "github_enabled": CONFIG.github_client_id.is_some(),
        "wechat_enabled": CONFIG.wechat_client_id.is_some(),
        "qq_enabled": CONFIG.qq_client_id.is_some(),
    }))
}

/// GET /api/auth/{provider}/start
#[get("/auth/{provider}/start")]
async fn oauth_start(path: web::Path<String>) -> impl Responder {
    let provider_str = path.into_inner();
    let provider = match provider_str.parse::<OAuthProvider>() {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({"error": e}));
        }
    };

    match build_oauth_client(provider) {
        Ok((_, auth_url, pkce_verifier)) => {
            // 提取state（从auth_url中）
            let state = extract_state_from_url(&auth_url).unwrap_or_default();

            // 存储state
            {
                let mut store = OAUTH_STATE_STORE.lock().unwrap();
                store.insert(state.clone(), (provider_str, pkce_verifier));
            }

            HttpResponse::Ok().json(json!({
                "auth_url": auth_url,
                "state": state,
            }))
        }
        Err(e) => {
            log::error!("OAuth start failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": e}))
        }
    }
}

/// GET /api/auth/{provider}/callback?code=...&state=...
#[get("/auth/{provider}/callback")]
async fn oauth_callback(
    path: web::Path<String>,
    query: web::Query<CallbackQuery>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let provider_str = path.into_inner();
    let code = &query.code;
    let state = &query.state;

    // 查找并移除state
    let (stored_provider, pkce_verifier) = {
        let mut store = OAUTH_STATE_STORE.lock().unwrap();
        match store.remove(state) {
            Some(data) => data,
            None => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid or expired state"}));
            }
        }
    };

    // 验证provider一致
    if stored_provider != provider_str {
        return HttpResponse::BadRequest().json(json!({"error": "Provider mismatch"}));
    }

    let provider = match provider_str.parse::<OAuthProvider>() {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({"error": e}));
        }
    };

    // 交换token并获取用户资料
    let profile = match exchange_code_and_get_user(provider, code, &pkce_verifier).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("OAuth callback failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error": e}));
        }
    };

    // 查找或创建用户
    let user_id = match find_or_create_user(&pool, &profile).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error": "Database error"}));
        }
    };

    // 生成JWT
    let token = match create_token(&user_id.to_string()) {
        Ok(t) => t,
        Err(e) => {
            log::error!("JWT creation failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error": "Token creation failed"}));
        }
    };

    // 存储session
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    if let Err(e) = sqlx::query!(
        "INSERT INTO sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)",
        uuid::Uuid::new_v4(),
        user_id,
        &token,
        expires_at
    )
    .execute(pool.get_ref())
    .await
    {
        log::error!("Failed to store session: {}", e);
    }

    // 获取用户信息
    let user_row = sqlx::query!(
        "SELECT id, email, display_name, avatar_url FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    let user = match user_row {
        Ok(row) => UserResponse {
            id: row.id.to_string(),
            email: row.email,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "User not found"}));
        }
    };

    HttpResponse::Ok().json(LoginResponse { token, user })
}

/// POST /api/auth/logout
#[post("/auth/logout")]
async fn logout(
    claims: AuthClaims,
    pool: web::Data<PgPool>,
) -> impl Responder {
    // 删除用户的所有session
    if let Err(e) = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", claims.sub.parse::<uuid::Uuid>().unwrap_or_default())
        .execute(pool.get_ref())
        .await
    {
        log::error!("Failed to delete sessions: {}", e);
    }

    HttpResponse::Ok().json(json!({"message": "Logged out successfully"}))
}

/// GET /api/auth/me
#[get("/auth/me")]
async fn get_me(
    claims: AuthClaims,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match claims.sub.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid user ID"}));
        }
    };

    let row = sqlx::query!(
        "SELECT id, email, display_name, avatar_url FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some(user)) => {
            HttpResponse::Ok().json(UserResponse {
                id: user.id.to_string(),
                email: user.email,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "User not found"})),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn find_or_create_user(
    pool: &PgPool,
    profile: &super::OAuthUserInfo,
) -> Result<uuid::Uuid, sqlx::Error> {
    // 先通过oauth account查找
    let existing = sqlx::query!(
        "SELECT user_id FROM oauth_accounts WHERE provider = $1 AND provider_account_id = $2",
        profile.provider,
        profile.provider_account_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        // 更新access_token
        let _ = sqlx::query!(
            "UPDATE oauth_accounts SET access_token = $1, refresh_token = $2, expires_at = $3, updated_at = NOW() WHERE user_id = $4 AND provider = $5",
            profile.access_token,
            profile.refresh_token,
            profile.expires_at,
            row.user_id,
            profile.provider
        )
        .execute(pool)
        .await;

        return Ok(row.user_id);
    }

    // 通过email查找（如果email存在）
    if let Some(ref email) = profile.email {
        let by_email = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = by_email {
            // 关联OAuth账号到现有用户
            let _ = sqlx::query!(
                "INSERT INTO oauth_accounts (id, user_id, provider, provider_account_id, access_token, refresh_token, expires_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())",
                uuid::Uuid::new_v4(),
                row.id,
                profile.provider,
                profile.provider_account_id,
                profile.access_token,
                profile.refresh_token,
                profile.expires_at
            )
            .execute(pool)
            .await;

            return Ok(row.id);
        }
    }

    // 创建新用户
    let user_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, email, display_name, avatar_url, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        user_id,
        profile.email,
        profile.display_name,
        profile.avatar_url
    )
    .execute(pool)
    .await?;

    // 创建OAuth关联
    sqlx::query!(
        "INSERT INTO oauth_accounts (id, user_id, provider, provider_account_id, access_token, refresh_token, expires_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())",
        uuid::Uuid::new_v4(),
        user_id,
        profile.provider,
        profile.provider_account_id,
        profile.access_token,
        profile.refresh_token,
        profile.expires_at
    )
    .execute(pool)
    .await?;

    Ok(user_id)
}

fn extract_state_from_url(url: &str) -> Option<String> {
    let url_parsed = url::Url::parse(url).ok()?;
    url_parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
}
