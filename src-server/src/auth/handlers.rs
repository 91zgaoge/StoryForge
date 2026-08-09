//! Auth Handlers — Actix-web HTTP Handlers

use std::{collections::HashMap, sync::Mutex};

use actix_web::{get, post, web, HttpResponse, Responder};
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::PgPool;

use super::{
    jwt::{create_token, AuthClaims},
    oauth::{build_oauth_client, exchange_code_and_get_user},
    LoginResponse, OAuthProvider, UserResponse,
};

/// 内存存储：state -> (provider, pkce_verifier, invite, created)
static OAUTH_STATE_STORE: Lazy<
    Mutex<HashMap<String, (String, String, Option<String>, std::time::Instant)>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));

const OAUTH_STATE_TTL_SECS: u64 = 600; // 10 分钟

/// 桌面登录流程：dstate -> DesktopFlow
struct DesktopFlow {
    oauth_state: String,
    done: Option<(String, String)>, // (token, user_json)
    /// 类型化失败通道：gated 失败等错误码（如 invalid_or_used_invite /
    /// internal）
    failed: Option<String>,
    created: std::time::Instant,
}

static DESKTOP_FLOW_STORE: Lazy<Mutex<HashMap<String, DesktopFlow>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const DESKTOP_FLOW_TTL_SECS: u64 = 600; // 10 分钟

fn desktop_flow_register(dstate: &str, oauth_state: &str) {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    store.retain(|_, f| f.created.elapsed().as_secs() < DESKTOP_FLOW_TTL_SECS);
    store.insert(
        dstate.to_string(),
        DesktopFlow {
            oauth_state: oauth_state.to_string(),
            done: None,
            failed: None,
            created: std::time::Instant::now(),
        },
    );
}

fn desktop_flow_complete(oauth_state: &str, token: String, user_json: String) {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    for flow in store.values_mut() {
        if flow.oauth_state == oauth_state {
            flow.done = Some((token.clone(), user_json.clone()));
        }
    }
}

/// 标记桌面流程失败（gated 拒绝等），desktop-poll 将一次性返回 403 + 错误码
fn desktop_flow_fail(oauth_state: &str, code: &str) {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    for flow in store.values_mut() {
        if flow.oauth_state == oauth_state {
            flow.failed = Some(code.to_string());
        }
    }
}

/// 取出失败码（一次性，取后删除条目，与 done 语义一致）
fn desktop_flow_take_failed(dstate: &str) -> Option<String> {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    let flow = store.get(dstate)?;
    flow.failed.as_ref()?;
    store.remove(dstate).and_then(|f| f.failed)
}

fn desktop_flow_take_done(dstate: &str) -> Option<(String, String)> {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    let flow = store.get(dstate)?;
    if flow.created.elapsed().as_secs() >= DESKTOP_FLOW_TTL_SECS {
        store.remove(dstate);
        return None;
    }
    flow.done.as_ref()?;
    store.remove(dstate).and_then(|f| f.done)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(auth_config)
        .service(oauth_start)
        .service(oauth_callback)
        .service(desktop_poll)
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
async fn oauth_start(path: web::Path<String>, query: web::Query<StartQuery>) -> impl Responder {
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

            // 存储state（含可选邀请码）；照 DESKTOP_FLOW_STORE 范式顺带清理过期条目
            {
                let mut store = OAUTH_STATE_STORE.lock().unwrap();
                store.retain(|_, (_, _, _, created)| {
                    created.elapsed().as_secs() < OAUTH_STATE_TTL_SECS
                });
                store.insert(
                    state.clone(),
                    (
                        provider_str,
                        pkce_verifier,
                        query.invite.clone(),
                        std::time::Instant::now(),
                    ),
                );
            }

            // 桌面端流程：注册 dstate 映射并直接 302 到 provider 授权页
            if query.client.as_deref() == Some("desktop") {
                if let Some(dstate) = &query.dstate {
                    desktop_flow_register(dstate, &state);
                    return HttpResponse::Found()
                        .append_header(("Location", auth_url))
                        .finish();
                }
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
    let (stored_provider, pkce_verifier, invite, _created) = {
        let mut store = OAUTH_STATE_STORE.lock().unwrap();
        match store.remove(state) {
            Some(data) => data,
            None => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": "Invalid or expired state"}));
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

    // 查找或创建用户（内测期门控：新用户需有效邀请码，老用户免码）
    let user_id = match find_or_create_user_gated(&pool, &profile, invite).await {
        Ok(id) => id,
        Err(e) => {
            // RowNotFound = 邀请码无效/用满；其余（DB 故障等）= internal
            let code = match e {
                sqlx::Error::RowNotFound => "invalid_or_used_invite",
                _ => "internal",
            };
            log::warn!("Invite gate rejected registration: {} (code={})", e, code);
            // 桌面端流程：写入 failed 失败码（desktop-poll 一次性 403），并仍展示错误页
            let is_desktop = DESKTOP_FLOW_STORE
                .lock()
                .unwrap()
                .values()
                .any(|f| f.oauth_state == *state);
            if is_desktop {
                desktop_flow_fail(state, code);
                let (title, hint) = if code == "invalid_or_used_invite" {
                    ("邀请码无效或已使用", "请返回 StoryMoss 应用重新输入。")
                } else {
                    ("登录失败", "服务器内部错误，请返回 StoryMoss 应用重试。")
                };
                return HttpResponse::Forbidden()
                    .content_type("text/html; charset=utf-8")
                    .body(format!(
                        "<html><head><meta charset=\"utf-8\"><title>注册受限</title></head>\
                         <body style=\"font-family:sans-serif;text-align:center;padding-top:80px\">\
                         <h2>{}</h2><p>{}</p></body></html>",
                        title, hint
                    ));
            }
            if code == "internal" {
                return HttpResponse::InternalServerError().json(json!({"error": code}));
            }
            return HttpResponse::Forbidden().json(json!({"error": code}));
        }
    };

    // 生成JWT
    let token = match create_token(&user_id.to_string()) {
        Ok(t) => t,
        Err(e) => {
            log::error!("JWT creation failed: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Token creation failed"}));
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
        "SELECT id, email, display_name, avatar_url, role FROM users WHERE id = $1",
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
            role: row.role,
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "User not found"}));
        }
    };

    // 桌面端流程：完成 dstate 映射并展示成功页
    let user_json = serde_json::to_string(&user).unwrap_or_default();
    {
        let store = DESKTOP_FLOW_STORE.lock().unwrap();
        if store.values().any(|f| f.oauth_state == *state) {
            drop(store);
            desktop_flow_complete(state, token, user_json);
            return HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(
                    "<html><head><meta charset=\"utf-8\"><title>登录成功</title></head>\
                 <body style=\"font-family:sans-serif;text-align:center;padding-top:80px\">\
                 <h2>登录成功</h2><p>请返回 StoryMoss 应用，登录状态将自动同步。</p></body></html>",
                );
        }
    }

    HttpResponse::Ok().json(LoginResponse { token, user })
}

/// GET /api/auth/desktop-poll?dstate=...
#[get("/auth/desktop-poll")]
async fn desktop_poll(query: web::Query<DesktopPollQuery>) -> impl Responder {
    // 失败通道优先：gated 拒绝等 → 一次性 403 + 错误码（取后删除条目，与 done
    // 语义一致）
    if let Some(code) = desktop_flow_take_failed(&query.dstate) {
        return HttpResponse::Forbidden().json(json!({"error": code}));
    }
    match desktop_flow_take_done(&query.dstate) {
        Some((token, user_json)) => {
            let user: serde_json::Value = serde_json::from_str(&user_json).unwrap_or(json!({}));
            HttpResponse::Ok().json(json!({ "token": token, "user": user }))
        }
        None => {
            let exists = DESKTOP_FLOW_STORE
                .lock()
                .unwrap()
                .contains_key(&query.dstate);
            if exists {
                HttpResponse::Accepted().json(json!({"status": "pending"}))
            } else {
                HttpResponse::NotFound().json(json!({"error": "unknown or expired dstate"}))
            }
        }
    }
}

/// POST /api/auth/logout
#[post("/auth/logout")]
async fn logout(claims: AuthClaims, pool: web::Data<PgPool>) -> impl Responder {
    // 删除用户的所有session
    if let Err(e) = sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1",
        claims.sub.parse::<uuid::Uuid>().unwrap_or_default()
    )
    .execute(pool.get_ref())
    .await
    {
        log::error!("Failed to delete sessions: {}", e);
    }

    HttpResponse::Ok().json(json!({"message": "Logged out successfully"}))
}

/// GET /api/auth/me
#[get("/auth/me")]
async fn get_me(claims: AuthClaims, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match claims.sub.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid user ID"}));
        }
    };

    let row = sqlx::query!(
        "SELECT id, email, display_name, avatar_url, role FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some(user)) => HttpResponse::Ok().json(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            role: user.role,
        }),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "User not found"})),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct StartQuery {
    client: Option<String>,
    dstate: Option<String>,
    invite: Option<String>,
}

#[derive(serde::Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(serde::Deserialize)]
struct DesktopPollQuery {
    dstate: String,
}

/// 查找已有用户：先按 OAuth 账号，再按 email；命中即返回
/// user_id（含既有副作用）
async fn find_existing_user(
    pool: &PgPool,
    profile: &super::OAuthUserInfo,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
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

        return Ok(Some(row.user_id));
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

            return Ok(Some(row.id));
        }
    }

    Ok(None)
}

/// 内测期注册门控：老用户免码；新用户需有效邀请码。
/// 校验 + 建用户 + 占码计数在同一事务内，UPDATE affected-rows 原子判断（0 行 =
/// 无效/用满）。
async fn find_or_create_user_gated(
    pool: &PgPool,
    profile: &super::OAuthUserInfo,
    invite: Option<String>,
) -> Result<uuid::Uuid, sqlx::Error> {
    // 老用户（OAuth 账号已存在或 email 已存在）免码
    if let Some(id) = find_existing_user(pool, profile).await? {
        return Ok(id);
    }

    let code = invite.ok_or(sqlx::Error::RowNotFound)?;

    let mut tx = pool.begin().await?;

    // 原子占码：0 行 = 邀请码不存在/已用满/已作废（revoked_at 软删生效）
    let claimed = sqlx::query!(
        "UPDATE invite_codes SET used_count = used_count + 1 \
         WHERE code = $1 AND used_count < max_uses AND revoked_at IS NULL \
         RETURNING grant_pro_days",
        code
    )
    .fetch_optional(&mut *tx)
    .await?;

    let grant_days = match claimed {
        Some(row) => row.grant_pro_days,
        None => return Err(sqlx::Error::RowNotFound),
    };

    // 创建新用户
    let user_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, email, display_name, avatar_url, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        user_id,
        profile.email,
        profile.display_name,
        profile.avatar_url
    )
    .execute(&mut *tx)
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
    .execute(&mut *tx)
    .await?;

    // 邀请码附赠 Pro N 天
    if let Some(days) = grant_days {
        sqlx::query!(
            "INSERT INTO subscriptions (user_id, tier, expires_at, source) VALUES ($1, 'pro', $2, 'invite') \
             ON CONFLICT (user_id) DO NOTHING",
            user_id,
            chrono::Utc::now() + chrono::Duration::days(days as i64)
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(user_id)
}

fn extract_state_from_url(url: &str) -> Option<String> {
    let url_parsed = url::Url::parse(url).ok()?;
    url_parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile(
        email: &str,
        provider: &str,
        provider_account_id: &str,
    ) -> crate::auth::OAuthUserInfo {
        crate::auth::OAuthUserInfo {
            provider: provider.to_string(),
            provider_account_id: provider_account_id.to_string(),
            email: Some(email.to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            access_token: "dummy-access-token".to_string(),
            refresh_token: None,
            expires_at: None,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn new_user_requires_valid_invite(pool: PgPool) {
        // 无码 → 拒绝
        let profile = test_profile("new1@test.com", "github", "gh-new1");
        assert!(find_or_create_user_gated(&pool, &profile, None)
            .await
            .is_err());

        // 错码 → 拒绝
        assert!(
            find_or_create_user_gated(&pool, &profile, Some("NOPE".into()))
                .await
                .is_err()
        );

        // 有效码 → 成功，used_count +1
        sqlx::query!("INSERT INTO invite_codes (code) VALUES ('BETA-1')")
            .execute(&pool)
            .await
            .unwrap();
        let uid = find_or_create_user_gated(&pool, &profile, Some("BETA-1".into()))
            .await
            .unwrap();
        let used: i32 =
            sqlx::query_scalar!("SELECT used_count FROM invite_codes WHERE code = 'BETA-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(used, 1);

        // 一码一次：第二个新用户用同码 → 拒绝
        let profile2 = test_profile("new2@test.com", "github", "gh-new2");
        assert!(
            find_or_create_user_gated(&pool, &profile2, Some("BETA-1".into()))
                .await
                .is_err()
        );

        // 老用户（OAuth 账号已存在）免码
        let again = find_or_create_user_gated(&pool, &profile, None)
            .await
            .unwrap();
        assert_eq!(again, uid);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn existing_email_user_bypasses_invite_gate(pool: PgPool) {
        // 预置老用户：email 相同、无 oauth_accounts
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            user_id,
            "legacy@test.com"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 不带邀请码：email 命中免码路径，成功并关联 OAuth 账号
        let profile = test_profile("legacy@test.com", "google", "g-legacy-1");
        let uid = find_or_create_user_gated(&pool, &profile, None)
            .await
            .unwrap();
        assert_eq!(uid, user_id);

        let linked: Option<uuid::Uuid> = sqlx::query_scalar!(
            "SELECT user_id FROM oauth_accounts WHERE provider = 'google' AND provider_account_id = 'g-legacy-1'"
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(linked, Some(user_id));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn invite_with_grant_pro_days_creates_pro_subscription(pool: PgPool) {
        sqlx::query!("INSERT INTO invite_codes (code, grant_pro_days) VALUES ('VIP-1', 90)")
            .execute(&pool)
            .await
            .unwrap();
        let profile = test_profile("vip@t.com", "github", "gh-vip");
        let uid = find_or_create_user_gated(&pool, &profile, Some("VIP-1".into()))
            .await
            .unwrap();

        // 注：sqlx 0.8 的 query_as! 宏不接受元组类型（要求 struct path），改用 query!
        let row = sqlx::query!(
            "SELECT tier, expires_at FROM subscriptions WHERE user_id = $1",
            uid
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.tier, "pro");
        assert!(row.expires_at.unwrap() > chrono::Utc::now() + chrono::Duration::days(80));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn revoked_invite_is_rejected(pool: PgPool) {
        sqlx::query!("INSERT INTO invite_codes (code, revoked_at) VALUES ('DEAD-1', NOW())")
            .execute(&pool)
            .await
            .unwrap();
        let profile = test_profile("dead@t.com", "github", "gh-dead");
        assert!(
            find_or_create_user_gated(&pool, &profile, Some("DEAD-1".into()))
                .await
                .is_err()
        );
    }

    #[test]
    fn desktop_flow_pending_then_done_once() {
        let dstate = "test-dstate-1";
        desktop_flow_register(dstate, "oauth-state-1");
        assert!(desktop_flow_take_done(dstate).is_none()); // pending

        desktop_flow_complete("oauth-state-1", "tok".into(), r#"{"id":"u1"}"#.into());
        let done = desktop_flow_take_done(dstate);
        assert!(done.is_some());
        // 一次性：再取没了
        assert!(desktop_flow_take_done(dstate).is_none());
    }

    #[test]
    fn desktop_flow_unknown_dstate() {
        assert!(desktop_flow_take_done("nope").is_none());
        assert!(desktop_flow_take_failed("nope").is_none());
    }

    #[test]
    fn desktop_flow_failed_polls_403_once() {
        let dstate = "test-dstate-fail-1";
        desktop_flow_register(dstate, "oauth-state-fail-1");
        assert!(desktop_flow_take_failed(dstate).is_none()); // 未失败时无失败码

        desktop_flow_fail("oauth-state-fail-1", "invalid_or_used_invite");
        assert_eq!(
            desktop_flow_take_failed(dstate).as_deref(),
            Some("invalid_or_used_invite")
        );
        // 一次性：再取没了，条目已删除（done/pending 也不可用）
        assert!(desktop_flow_take_failed(dstate).is_none());
        assert!(desktop_flow_take_done(dstate).is_none());
    }
}
