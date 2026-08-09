# OAuth 登录绑定订阅 + 跨设备同步 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 订阅从设备 machine_id 绑定切换为 OAuth 登录账号，登录后 Pro 跨设备同步；未登录沿用 machine_id 降级。付款不在本期。

**Architecture:** src-server（Actix-web + Postgres）为订阅权威源，新增订阅 API 与桌面 OAuth 中转（302 + dstate 轮询）；桌面端登录后订阅身份切换为 server UUID，远程结果写本地缓存，所有本地订阅检查（has_feature_access 等 6 处）保持同步读本地缓存不变；前端打通已有登录 UI 与 UpgradeModal 登录引导。

**Tech Stack:** Actix-web 4 / sqlx 0.8(Postgres) / oauth2 5 / jsonwebtoken 9；Tauri 2 / rusqlite / reqwest 0.12；React 18 / zustand / vitest。

**设计文档：** `docs/plans/2026-08-09-oauth-subscription-sync-design.md`（已批准）

## Global Constraints

- 中文 conventional commit（如 `新增(server): ...`），提交不夹带 `.recovery/`
- 版本号 5 处同步仅出现在 Task 8：src-tauri/Cargo.toml、src-tauri/tauri.conf.json、src-frontend/package.json、landing/src/hooks/useLatestRelease.ts(FALLBACK_VERSION)、Cargo.lock storymoss 条目
- pre-commit 钩子：`cargo +nightly fmt` + prettier 必须通过
- OAuth 提供商仅 Google + GitHub；微信/QQ 预留不动
- server 编译/测试需要 `DATABASE_URL`（sqlx 编译期校验）：先 `docker compose up -d postgres`，并在 `src-server/.env` 写 `DATABASE_URL=postgres://storymoss:changeme@localhost:5432/storymoss`（确认 .env 被 gitignore）
- 桌面 OAuth 不起本地 HTTP 监听、不用深链接：server 中转 + 轮询
- 账本 `.superpowers/sdd/progress.md` 每个 Task 完成后追加一行

---

### Task 1: Server 订阅 API

**Files:**
- Create: `src-server/migrations/002_subscriptions.sql`
- Create: `src-server/src/api/subscription.rs`
- Modify: `src-server/src/api/mod.rs`
- Modify: `src-server/src/config.rs:8-25`（加字段）

**Interfaces:**
- Consumes: `crate::auth::jwt::AuthClaims`（FromRequest 提取器，`sub: String`）；`sqlx::PgPool`
- Produces:
  - `GET /api/subscription/me`（Bearer JWT）→ `SubscriptionResponse { user_id: String, tier: String, status: String, expires_at: Option<String> }`（expires_at 为 RFC3339）；无记录自动建 free
  - `POST /api/subscription/dev-upgrade`，body `{"tier":"pro"|"free"|"enterprise"}` → SubscriptionResponse；pro 给 30 天 expires；env `DEV_UPGRADE_ENABLED=false` 时 403
  - `pub struct SubscriptionResponse`（serde Serialize，Task 3 的桌面端按此 JSON 解析）

- [ ] **Step 1: 写迁移**

`src-server/migrations/002_subscriptions.sql`：

```sql
-- 订阅表（server 为权威源；桌面端本地表仅作缓存）

CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    tier TEXT NOT NULL DEFAULT 'free',
    status TEXT NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    source TEXT NOT NULL DEFAULT 'dev',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON subscriptions(user_id);
```

- [ ] **Step 2: config.rs 加开关**

`ServerConfig` struct 加字段 `pub dev_upgrade_enabled: bool,`，`from_env` 加：

```rust
dev_upgrade_enabled: env::var("DEV_UPGRADE_ENABLED")
    .map(|v| v != "false")
    .unwrap_or(true),
```

- [ ] **Step 3: 写失败测试**

`src-server/src/api/subscription.rs` 先只放测试模块（handler 随后补）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn me_creates_free_subscription_for_new_user(pool: PgPool) {
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", user_id, "t@t.com")
            .execute(&pool).await.unwrap();

        let status = get_or_create(&pool, user_id).await.unwrap();
        assert_eq!(status.tier, "free");
        assert_eq!(status.status, "active");
        assert_eq!(status.expires_at, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn dev_upgrade_sets_pro_with_expiry(pool: PgPool) {
        let user_id = uuid::Uuid::new_v4();
        sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", user_id, "t@t.com")
            .execute(&pool).await.unwrap();

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
```

运行：`cd src-server && docker compose -f ../docker-compose.yml up -d postgres && DATABASE_URL=postgres://storymoss:changeme@localhost:5432/storymoss cargo test subscription` — 预期 FAIL（函数不存在）。

- [ ] **Step 4: 实现 handler**

`src-server/src/api/subscription.rs` 顶部（测试之上）：

```rust
//! Subscription API — server 为订阅权威源

use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;

use crate::auth::jwt::AuthClaims;
use crate::config::CONFIG;

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
        return Ok(SubscriptionRow { tier: row.tier, status: row.status, expires_at: row.expires_at });
    }

    sqlx::query!("INSERT INTO subscriptions (user_id) VALUES ($1)", user_id)
        .execute(pool)
        .await?;
    Ok(SubscriptionRow { tier: "free".into(), status: "active".into(), expires_at: None })
}

async fn upsert_tier(pool: &PgPool, user_id: uuid::Uuid, tier: &str) -> Result<SubscriptionRow, sqlx::Error> {
    let expires_at = if tier == "pro" {
        Some(chrono::Utc::now() + chrono::Duration::days(30))
    } else {
        None
    };
    sqlx::query!(
        "INSERT INTO subscriptions (user_id, tier, expires_at) VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE SET tier = $2, expires_at = $3, updated_at = NOW()",
        user_id, tier, expires_at
    )
    .execute(pool)
    .await?;
    Ok(SubscriptionRow { tier: tier.into(), status: "active".into(), expires_at })
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
```

`src-server/src/api/mod.rs`：`mod subscription;` 并在 scope 链加 `.configure(subscription::init_routes)`。

- [ ] **Step 5: 测试通过 + 编译**

`DATABASE_URL=... cargo test subscription` 全过；`cargo check`（同 env）无错。

- [ ] **Step 6: Commit**

```bash
git add src-server/migrations/002_subscriptions.sql src-server/src/api/subscription.rs src-server/src/api/mod.rs src-server/src/config.rs
git commit -m "新增(server): 订阅 API（GET /subscription/me + POST /subscription/dev-upgrade）"
```

---

### Task 2: Server 桌面 OAuth 中转（302 + dstate 轮询）

**Files:**
- Modify: `src-server/src/auth/handlers.rs`

**Interfaces:**
- Consumes: Task 1 不动；现有 `OAUTH_STATE_STORE`、`LoginResponse`、`UserResponse`
- Produces:
  - `GET /api/auth/{provider}/start?client=desktop&dstate=<uuid>` → 302 到 provider 授权页（无 client 参数时保持原 JSON 行为，web 端不受影响）
  - `GET /api/auth/desktop-poll?dstate=<uuid>` → `202 {"status":"pending"}`；登录完成后一次性返回 `200 LoginResponse { token, user }` 并删除记录；dstate 未知 → 404
  - callback 中 client=desktop 时返回 HTML「登录成功，请返回 StoryMoss」页（不再返回 JSON）

- [ ] **Step 1: 写失败测试（store 逻辑）**

handlers.rs 底部测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
```

运行 `DATABASE_URL=... cargo test auth::handlers` — FAIL。

- [ ] **Step 2: 实现**

handlers.rs 顶部静态区加：

```rust
/// 桌面登录流程：dstate -> DesktopFlow
struct DesktopFlow {
    oauth_state: String,
    done: Option<(String, String)>, // (token, user_json)
    created: std::time::Instant,
}

static DESKTOP_FLOW_STORE: Lazy<Mutex<HashMap<String, DesktopFlow>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const DESKTOP_FLOW_TTL_SECS: u64 = 600; // 10 分钟

fn desktop_flow_register(dstate: &str, oauth_state: &str) {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    store.retain(|_, f| f.created.elapsed().as_secs() < DESKTOP_FLOW_TTL_SECS);
    store.insert(dstate.to_string(), DesktopFlow {
        oauth_state: oauth_state.to_string(),
        done: None,
        created: std::time::Instant::now(),
    });
}

fn desktop_flow_complete(oauth_state: &str, token: String, user_json: String) {
    let mut store = DESKTOP_FLOW_STORE.lock().unwrap();
    for flow in store.values_mut() {
        if flow.oauth_state == oauth_state {
            flow.done = Some((token.clone(), user_json.clone()));
        }
    }
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
```

`init_routes` 加 `.service(desktop_poll)`。

`oauth_start` 改为接受 query：

```rust
#[derive(serde::Deserialize)]
struct StartQuery {
    client: Option<String>,
    dstate: Option<String>,
}
```

签名加 `query: web::Query<StartQuery>`；存 state 后：

```rust
if query.client.as_deref() == Some("desktop") {
    if let Some(dstate) = &query.dstate {
        desktop_flow_register(dstate, &state);
        return HttpResponse::Found()
            .append_header(("Location", auth_url))
            .finish();
    }
}
// 原 JSON 返回保持不变
```

`oauth_callback` 末尾（拿到 `LoginResponse { token, user }` 后）改为：

```rust
// 桌面端流程：完成 dstate 映射并展示成功页
let user_json = serde_json::to_string(&user).unwrap_or_default();
{
    let store = DESKTOP_FLOW_STORE.lock().unwrap();
    if store.values().any(|f| f.oauth_state == *state) {
        drop(store);
        desktop_flow_complete(state, token, user_json);
        return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
            "<html><head><meta charset=\"utf-8\"><title>登录成功</title></head>\
             <body style=\"font-family:sans-serif;text-align:center;padding-top:80px\">\
             <h2>登录成功</h2><p>请返回 StoryMoss 应用，登录状态将自动同步。</p></body></html>",
        );
    }
}

HttpResponse::Ok().json(LoginResponse { token, user })
```

新 endpoint：

```rust
/// GET /api/auth/desktop-poll?dstate=...
#[get("/auth/desktop-poll")]
async fn desktop_poll(query: web::Query<DesktopPollQuery>) -> impl Responder {
    match desktop_flow_take_done(&query.dstate) {
        Some((token, user_json)) => {
            let user: serde_json::Value = serde_json::from_str(&user_json).unwrap_or(json!({}));
            HttpResponse::Ok().json(json!({ "token": token, "user": user }))
        }
        None => {
            let exists = DESKTOP_FLOW_STORE.lock().unwrap().contains_key(&query.dstate);
            if exists {
                HttpResponse::Accepted().json(json!({"status": "pending"}))
            } else {
                HttpResponse::NotFound().json(json!({"error": "unknown or expired dstate"}))
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct DesktopPollQuery {
    dstate: String,
}
```

注意：callback 里原来 `HttpResponse::Ok().json(LoginResponse { token, user })` 的 token/user 被移动，上面的桌面分支要放在 JSON 返回之前且把 `user` 先序列化。`state` 变量原为 `&String`，`desktop_flow_complete(state, ...)` 直接传。

- [ ] **Step 3: 测试通过**

`DATABASE_URL=... cargo test auth::handlers` 全过；`cargo check` 无错。

- [ ] **Step 4: Commit**

```bash
git add src-server/src/auth/handlers.rs
git commit -m "新增(server): 桌面 OAuth 中转（302 授权 + dstate 一次性轮询换 JWT）"
```

---

### Task 2B: Server 邀请码注册门控

**Files:**
- Create: `src-server/migrations/003_invite_codes.sql`
- Modify: `src-server/src/auth/handlers.rs`

**Interfaces:**
- Consumes: Task 2 的 `OAUTH_STATE_STORE`（值扩为含 invite）、`DESKTOP_FLOW_STORE`、`find_or_create_user`
- Produces:
  - `GET /api/auth/{provider}/start` 增加可选 query `invite=<code>`（desktop/web 通用），随 state 存入 `OAUTH_STATE_STORE`
  - callback 门控：OAuth 账号已存在或 email 已存在 → 老用户免码；否则校验 invite——缺失/不存在/已用满 → `403 {"error":"invalid_or_used_invite"}`（desktop 流程则展示错误 HTML 页）；有效 → 建用户并在同一事务 `used_count + 1`
  - 邀请码发放：管理员 SQL 手工 `INSERT INTO invite_codes (code, max_uses, note) VALUES (...)`

- [ ] **Step 1: 写迁移**

```sql
-- 邀请码（内测期注册门控）

CREATE TABLE IF NOT EXISTS invite_codes (
    code TEXT PRIMARY KEY,
    max_uses INT NOT NULL DEFAULT 1,
    used_count INT NOT NULL DEFAULT 0,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- [ ] **Step 2: 写失败测试**

handlers.rs 测试模块加（`#[sqlx::test(migrations = "./migrations")]`）：

```rust
#[sqlx::test(migrations = "./migrations")]
async fn new_user_requires_valid_invite(pool: PgPool) {
    // 无码 → 拒绝
    let profile = test_profile("new1@test.com", "github", "gh-new1");
    assert!(find_or_create_user_gated(&pool, &profile, None).await.is_err());

    // 错码 → 拒绝
    assert!(find_or_create_user_gated(&pool, &profile, Some("NOPE".into())).await.is_err());

    // 有效码 → 成功，used_count +1
    sqlx::query!("INSERT INTO invite_codes (code) VALUES ('BETA-1')").execute(&pool).await.unwrap();
    let uid = find_or_create_user_gated(&pool, &profile, Some("BETA-1".into())).await.unwrap();
    let used: i32 = sqlx::query_scalar!("SELECT used_count FROM invite_codes WHERE code = 'BETA-1'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(used, 1);

    // 一码一次：第二个新用户用同码 → 拒绝
    let profile2 = test_profile("new2@test.com", "github", "gh-new2");
    assert!(find_or_create_user_gated(&pool, &profile2, Some("BETA-1".into())).await.is_err());

    // 老用户（OAuth 账号已存在）免码
    let again = find_or_create_user_gated(&pool, &profile, None).await.unwrap();
    assert_eq!(again, uid);
}
```

`test_profile` 为构造 `OAuthUserInfo` 的测试辅助。`find_or_create_user_gated(pool, profile, invite: Option<String>) -> Result<Uuid, sqlx::Error>`：内部先走原 `find_or_create_user` 的「查找」两段（oauth 账号、email），命中即返回；未命中才校验 invite 并建用户。校验+建用户+计数在一个事务里（`pool.begin()`），用 `UPDATE invite_codes SET used_count = used_count + 1 WHERE code = $1 AND used_count < max_uses` 的 affected-rows 判断原子占码（0 行 = 无效/用满）。

运行 `cargo test auth::handlers` — FAIL。

- [ ] **Step 3: 实现门控与接线**

- `StartQuery` 加 `invite: Option<String>`；`OAUTH_STATE_STORE` 值类型从 `(String, String)` 扩为 `(String, String, Option<String>)`（provider, pkce_verifier, invite），callback 取出后传入 gated 函数
- callback 调 `find_or_create_user` 处换成 `find_or_create_user_gated(&pool, &profile, invite).await`，错误分支：
  - desktop 流程（state 在 DESKTOP_FLOW_STORE 中）：返回错误 HTML「邀请码无效或已使用，请返回应用重新输入」
  - 其余：`403 {"error":"invalid_or_used_invite"}`
- 注意 Task 2 的 `desktop_flow_complete` 不受失败影响（失败时不 complete，桌面轮询继续 pending 直至过期——前端有 2 分钟超时兜底）

- [ ] **Step 4: 测试通过 + Commit**

`cargo test auth::handlers` 全过。

```bash
git add src-server/migrations/003_invite_codes.sql src-server/src/auth/handlers.rs
git commit -m "新增(server): 邀请码注册门控（新用户注册需有效邀请码，老用户免码）"
```

---

### Task 3: 桌面端 server_client + auth 命令重写

**Files:**
- Create: `src-tauri/src/server_client.rs`
- Modify: `src-tauri/src/lib.rs`（`mod server_client;`）
- Modify: `src-tauri/Cargo.toml`（加 `urlencoding = "2"`——若不想加依赖，可手写仅对邀请码常见字符的极简 percent-encode 辅助函数代替）
- Modify: `src-tauri/src/auth/commands.rs`（重写 oauth_start / get_current_user / logout，新增 oauth_poll_login）
- Modify: `src-tauri/src/db/repositories/user_repository.rs`（加方法）
- Modify: `src-tauri/src/handlers.rs:254-258`（注册 oauth_poll_login）

**Interfaces:**
- Consumes: Task 1/2 的 server API；现有 `UserRepository`、`crate::db::UserInfo`
- Produces（Task 4/5 依赖）:
  - `server_client::ServerClient::new() -> Self`（base 读 env `STORYMOSS_SERVER_URL`，默认 `https://storymoss.top`）
  - `ServerClient::desktop_auth_url(&self, provider: &str, dstate: &str, invite: Option<&str>) -> String`（invite 非空时拼 `&invite=`，URL-encode）
  - `ServerClient::desktop_poll(&self, dstate: &str) -> Result<Option<DesktopLogin>, AppError>`（None=pending；404/网络错按 Err 上抛由调用方区分——404 视为过期返回 Err）
  - `ServerClient::get_subscription(&self, token: &str) -> Result<RemoteSubscription, AppError>`
  - `ServerClient::dev_upgrade(&self, token: &str, tier: &str) -> Result<RemoteSubscription, AppError>`
  - `ServerClient::logout(&self, token: &str) -> Result<(), AppError>`
  - `DesktopLogin { token: String, user_id: String, email: Option<String>, display_name: Option<String>, avatar_url: Option<String> }`
  - `RemoteSubscription { tier: String, status: String, expires_at: Option<String> }`
  - Tauri 命令：`oauth_start(provider, invite?) -> { auth_url, dstate }`；`oauth_poll_login(dstate) -> Option<CurrentSession>`；`get_current_user() -> Option<CurrentSession>`；`logout(token)`
  - `CurrentSession { user: crate::db::UserInfo, token: String }`（serde Serialize）
  - `UserRepository::upsert_server_user(&self, id: &str, email: Option<String>, display_name: Option<String>, avatar_url: Option<String>) -> Result<User, _>`
  - `UserRepository::find_latest_valid_session(&self) -> Option<(User, String)>`

- [ ] **Step 1: server_client.rs + 单元测试**

```rust
//! StoryMoss Server HTTP 客户端（订阅权威源 + 桌面 OAuth 中转）

use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

const DEFAULT_BASE_URL: &str = "https://storymoss.top";

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteSubscription {
    pub tier: String,
    pub status: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DesktopLogin {
    pub token: String,
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct PollResponse {
    token: String,
    user: PollUser,
}

#[derive(Deserialize)]
struct PollUser {
    id: String,
    email: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
}

pub struct ServerClient {
    base_url: String,
    client: Client,
}

impl ServerClient {
    pub fn new() -> Self {
        let base_url = std::env::var("STORYMOSS_SERVER_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self { base_url: base_url.trim_end_matches('/').to_string(), client: Client::new() }
    }

    pub fn desktop_auth_url(&self, provider: &str, dstate: &str, invite: Option<&str>) -> String {
        let mut url = format!("{}/api/auth/{}/start?client=desktop&dstate={}", self.base_url, provider, dstate);
        if let Some(code) = invite.filter(|c| !c.trim().is_empty()) {
            url.push_str(&format!("&invite={}", urlencoding::encode(code.trim())));
        }
        url
    }

    /// Ok(None) = pending；Err = 过期或网络失败
    pub async fn desktop_poll(&self, dstate: &str) -> Result<Option<DesktopLogin>, AppError> {
        let resp = self
            .client
            .get(format!("{}/api/auth/desktop-poll", self.base_url))
            .query(&[("dstate", dstate)])
            .send()
            .await
            .map_err(|e| format!("desktop poll failed: {}", e))?;
        match resp.status().as_u16() {
            202 => Ok(None),
            200 => {
                let body: PollResponse = resp.json().await.map_err(|e| format!("bad poll body: {}", e))?;
                Ok(Some(DesktopLogin {
                    token: body.token,
                    user_id: body.user.id,
                    email: body.user.email,
                    display_name: body.user.display_name,
                    avatar_url: body.user.avatar_url,
                }))
            }
            s => Err(format!("desktop poll status {}", s).into()),
        }
    }

    pub async fn get_subscription(&self, token: &str) -> Result<RemoteSubscription, AppError> {
        let resp = self
            .client
            .get(format!("{}/api/subscription/me", self.base_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("get subscription failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("subscription status {}", resp.status()).into());
        }
        resp.json().await.map_err(|e| format!("bad subscription body: {}", e).into())
    }

    pub async fn dev_upgrade(&self, token: &str, tier: &str) -> Result<RemoteSubscription, AppError> {
        let resp = self
            .client
            .post(format!("{}/api/subscription/dev-upgrade", self.base_url))
            .bearer_auth(token)
            .json(&serde_json::json!({"tier": tier}))
            .send()
            .await
            .map_err(|e| format!("dev upgrade failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("dev upgrade status {}", resp.status()).into());
        }
        resp.json().await.map_err(|e| format!("bad upgrade body: {}", e).into())
    }

    pub async fn logout(&self, token: &str) -> Result<(), AppError> {
        let _ = self
            .client
            .post(format!("{}/api/auth/logout", self.base_url))
            .bearer_auth(token)
            .send()
            .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_auth_url_format() {
        let c = ServerClient::new();
        assert_eq!(
            c.desktop_auth_url("google", "abc", None),
            "https://storymoss.top/api/auth/google/start?client=desktop&dstate=abc"
        );
        assert_eq!(
            c.desktop_auth_url("google", "abc", Some("BETA 1")),
            "https://storymoss.top/api/auth/google/start?client=desktop&dstate=abc&invite=BETA%201"
        );
    }

    #[test]
    fn remote_subscription_parses_server_json() {
        let s: RemoteSubscription =
            serde_json::from_str(r#"{"user_id":"u","tier":"pro","status":"active","expires_at":null}"#)
                .unwrap();
        assert_eq!(s.tier, "pro");
        assert_eq!(s.expires_at, None);
    }
}
```

`src-tauri/src/lib.rs` 加 `pub mod server_client;`。运行 `cd src-tauri && cargo test server_client` 通过。

- [ ] **Step 2: UserRepository 加方法 + 测试**

先看 `src-tauri/src/db/repositories/user_repository.rs` 现有 `create_user`/`create_session` 的列与返回类型，按同风格加：

```rust
/// 写入/更新 server 登录用户（id 为 server UUID）
pub fn upsert_server_user(
    &self,
    id: &str,
    email: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
) -> Result<User, rusqlite::Error> {
    let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO users (id, email, display_name, avatar_url, is_local_user, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)
         ON CONFLICT(id) DO UPDATE SET email=?2, display_name=?3, avatar_url=?4, updated_at=?5",
        params![id, email, display_name, avatar_url, now],
    )?;
    drop(conn);
    // 复用现有按 id 查询逻辑（若无 find_by_id 则顺手加）
    self.find_by_id(id)
}

/// 取最新未过期 session 及其用户（get_current_user 用）
pub fn find_latest_valid_session(&self) -> Option<(User, String)> {
    let conn = self.pool.get().ok()?;
    let now = chrono::Local::now().to_rfc3339();
    conn.query_row(
        "SELECT u.id, u.email, u.display_name, u.avatar_url, u.is_local_user, s.token
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.expires_at > ?1 ORDER BY s.created_at DESC LIMIT 1",
        params![now],
        |row| Ok((User {
            id: row.get(0)?, email: row.get(1)?, display_name: row.get(2)?,
            avatar_url: row.get(3)?, is_local_user: row.get::<_, i64>(4)? != 0,
        }, row.get::<_, String>(5)?)),
    )
    .optional()
    .ok()
    .flatten()
}
```

（实现时以 user_repository.rs 现有 `User` 结构体字段与 `create_session(token, expires_at)` 签名为准微调；sessions 表有 expires_at/created_at 列——若本地 V033 的 sessions 表列不同，以 V033 实际 DDL 为准调整 SQL。）

测试（create_test_pool 跑全部迁移，参考 V121 测试写法）：upsert 新建→find_latest_valid_session 命中；session 过期→None。

- [ ] **Step 3: 重写 auth/commands.rs**

```rust
//! Tauri IPC Commands — 认证（server 中转型）

use tauri::{AppHandle, State};

use super::OAuthProvider;
use crate::{
    db::{DbPool, UserInfo, UserRepository},
    error::AppError,
    server_client::ServerClient,
};

#[derive(Debug, serde::Serialize)]
pub struct CurrentSession {
    pub user: UserInfo,
    pub token: String,
}

/// 获取当前认证配置（登录入口恒指向 server，始终可用）
#[tauri::command]
pub fn get_auth_config() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({
        "google_enabled": true,
        "github_enabled": true,
        "wechat_enabled": false,
        "qq_enabled": false,
    }))
}

/// 开始 OAuth 登录：返回 server 授权 URL 与 dstate，前端打开浏览器后轮询 oauth_poll_login
/// invite 为可选邀请码（内测期新用户注册需要，见 Task 2B）
#[tauri::command]
pub fn oauth_start(provider: String, invite: Option<String>) -> Result<serde_json::Value, AppError> {
    let provider = provider.parse::<OAuthProvider>().map_err(AppError::from)?;
    let dstate = uuid::Uuid::new_v4().to_string();
    let client = ServerClient::new();
    Ok(serde_json::json!({
        "auth_url": client.desktop_auth_url(&provider.to_string(), &dstate, invite.as_deref()),
        "dstate": dstate,
    }))
}

/// 轮询一次桌面登录结果；成功则落本地用户+session 并返回
#[tauri::command]
pub async fn oauth_poll_login(
    dstate: String,
    app_handle: AppHandle,
) -> Result<Option<CurrentSession>, AppError> {
    let client = ServerClient::new();
    let login = match client.desktop_poll(&dstate).await? {
        Some(l) => l,
        None => return Ok(None),
    };

    let pool = app_handle.state::<DbPool>().inner().clone();
    let repo = UserRepository::new(pool);
    let user = repo
        .upsert_server_user(&login.user_id, login.email, login.display_name, login.avatar_url)
        .map_err(AppError::from)?;
    let expires_at = chrono::Local::now() + chrono::Duration::days(7);
    repo.create_session(&user.id, &login.token, expires_at)
        .map_err(AppError::from)?;

    // 登录后立即同步一次订阅缓存
    crate::subscription::sync_remote_subscription(&app_handle);

    Ok(Some(CurrentSession { user: repo.to_user_info(&user), token: login.token }))
}

/// 获取当前登录会话（启动恢复用）
#[tauri::command]
pub fn get_current_user(pool: State<'_, DbPool>) -> Result<Option<CurrentSession>, AppError> {
    let repo = UserRepository::new(pool.inner().clone());
    Ok(repo
        .find_latest_valid_session()
        .map(|(user, token)| CurrentSession { user: repo.to_user_info(&user), token }))
}

/// 注销：删本地 session + 尽力通知 server
#[tauri::command]
pub async fn logout(token: String, pool: State<'_, DbPool>) -> Result<(), AppError> {
    let repo = UserRepository::new(pool.inner().clone());
    repo.delete_session(&token).map_err(AppError::from)?;
    let _ = ServerClient::new().logout(&token).await;
    Ok(())
}
```

旧 `oauth_callback` 命令删除；`handlers.rs` 注册表把 `auth::commands::oauth_callback` 换成 `auth::commands::oauth_poll_login`（其余名字不变）。`auth/oauth.rs` 不再被命令引用，加 `#[allow(dead_code)]` 或保留（后续本地监听可能复用）——选择保留并确保 `cargo check` 无 warning-as-error。

- [ ] **Step 4: 编译 + 测试**

`cd src-tauri && cargo test auth::` + `cargo test user_repository` 全过；`cargo check` 无错。`crate::subscription::sync_remote_subscription` 此时还不存在——先加占位（Task 4 实装）：

```rust
// src-tauri/src/subscription/mod.rs 末尾（Task 4 替换为实现）
pub fn sync_remote_subscription(_app_handle: &tauri::AppHandle) {}
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/server_client.rs src-tauri/src/lib.rs src-tauri/src/auth/commands.rs src-tauri/src/db/repositories/user_repository.rs src-tauri/src/handlers.rs src-tauri/src/subscription/mod.rs
git commit -m "新增(桌面): server_client + auth 命令改走 server 中转登录（轮询换 JWT）"
```

---

### Task 4: 订阅身份收口 + 远程同步缓存

**Files:**
- Create: `src-tauri/src/subscription/identity.rs`
- Modify: `src-tauri/src/subscription/mod.rs`（`pub mod identity;` + `sync_remote_subscription` 实装 + `cache_remote_status`）
- Modify: `src-tauri/src/subscription/commands.rs`（命令 async 化、远程优先）
- Modify: 其余 5 处 `get_user_id` 复制点：`src-tauri/src/agents/commands.rs`、`src-tauri/src/pipeline/commands.rs`、`src-tauri/src/book_deconstruction/commands.rs`、`src-tauri/src/guidebook_distillation/commands.rs`、`src-tauri/src/llm/service.rs`（及 `src-tauri/src/agents/service.rs` 若有）

**Interfaces:**
- Consumes: Task 3 的 `ServerClient`、`UserRepository::find_latest_valid_session`
- Produces:
  - `identity::resolve_identity(app_handle, pool) -> Identity`
  - `pub enum Identity { Account { user_id: String, token: String }, Device { machine_id: String } }`
  - `identity::resolve_user_id(app_handle, pool) -> String`（同步，供 6 处现有同步检查点替换 `get_user_id`）
  - `subscription::sync_remote_subscription(app_handle)`（后台 spawn，远程拉取写本地缓存）
  - `subscription::cache_remote_status(pool, user_id, &RemoteSubscription)`

- [ ] **Step 1: identity.rs + 测试**

```rust
//! 订阅身份解析：已登录 = server 账号 UUID；未登录 = 设备 machine_id

use tauri::{AppHandle, Manager};

use crate::db::{DbPool, UserRepository};

#[derive(Debug, Clone, PartialEq)]
pub enum Identity {
    Account { user_id: String, token: String },
    Device { machine_id: String },
}

/// 设备标识（原 subscription/commands.rs get_user_id 逻辑移此，全仓唯一出处）
pub fn machine_id(app_handle: &AppHandle) -> String {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
    let path = app_dir.join(".machine_id");
    if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default().trim().to_string()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = std::fs::create_dir_all(&app_dir);
        let _ = std::fs::write(&path, &id);
        id
    }
}

pub fn resolve_identity(app_handle: &AppHandle, pool: &DbPool) -> Identity {
    let repo = UserRepository::new(pool.clone());
    if let Some((user, token)) = repo.find_latest_valid_session() {
        return Identity::Account { user_id: user.id, token };
    }
    Identity::Device { machine_id: machine_id(app_handle) }
}

/// 现有 6 处同步订阅检查点的 user_id 来源（读本地缓存，无网络）
pub fn resolve_user_id(app_handle: &AppHandle, pool: &DbPool) -> String {
    match resolve_identity(app_handle, pool) {
        Identity::Account { user_id, .. } => user_id,
        Identity::Device { machine_id } => machine_id,
    }
}
```

测试：无 session → Device；插入有效 session（用 Task 3 的 upsert_server_user + create_session）→ Account。machine_id 测试用 tempfile 目录构造 AppHandle 太重——只测纯函数部分（写/读临时文件），参考现有测试有无 AppHandle mock；若无，machine_id 逻辑并入 identity 后由集成手测覆盖，单测聚焦 resolve_identity。

- [ ] **Step 2: mod.rs 加缓存与同步**

```rust
pub mod identity;

/// 远程订阅状态写本地缓存（供 has_feature_access 等同步检查点读取）
pub fn cache_remote_status(
    pool: &DbPool,
    user_id: &str,
    remote: &crate::server_client::RemoteSubscription,
) -> Result<(), AppError> {
    let service = SubscriptionService::new(pool.clone());
    let current = service.get_or_create_subscription(user_id)?;
    if current.tier != remote.tier {
        let expires_days = if remote.tier == "pro" { Some(30) } else { None };
        service.upgrade_subscription(user_id, &remote.tier, expires_days)?;
    }
    Ok(())
}

/// 已登录时后台同步一次远程订阅（登录后/启动时调用）
pub fn sync_remote_subscription(app_handle: &tauri::AppHandle) {
    use tauri::Manager;
    let pool = app_handle.state::<DbPool>().inner().clone();
    let identity = identity::resolve_identity(app_handle, &pool);
    if let identity::Identity::Account { user_id, token } = identity {
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let client = crate::server_client::ServerClient::new();
            match client.get_subscription(&token).await {
                Ok(remote) => {
                    if let Err(e) = cache_remote_status(&pool, &user_id, &remote) {
                        log::warn!("cache remote subscription failed: {}", e);
                    }
                    let _ = crate::state_sync::StateSync::emit_subscription_changed(
                        &handle, &user_id, &remote.tier,
                    );
                }
                Err(e) => log::warn!("remote subscription sync failed (offline ok): {}", e),
            }
        });
    }
}
```

（`SubscriptionStatus` 与 `upgrade_subscription` 行为见 mod.rs 现状；若 crate 无 `log` 依赖用 `eprintln!` 或现有 logger 宏，以仓库现状为准。）

- [ ] **Step 3: commands.rs 远程优先**

```rust
#[command]
pub async fn get_subscription_status(app_handle: AppHandle) -> Result<SubscriptionStatus, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let service = SubscriptionService::new(pool.clone());
    match identity::resolve_identity(&app_handle, &pool) {
        identity::Identity::Account { user_id, token } => {
            let client = crate::server_client::ServerClient::new();
            match client.get_subscription(&token).await {
                Ok(remote) => {
                    let _ = super::cache_remote_status(&pool, &user_id, &remote);
                    Ok(SubscriptionStatus {
                        user_id,
                        tier: remote.tier,
                        status: remote.status,
                        expires_at: remote.expires_at,
                    })
                }
                Err(_) => service.get_or_create_subscription(&user_id), // 离线走缓存
            }
        }
        identity::Identity::Device { machine_id } => service.get_or_create_subscription(&machine_id),
    }
}

#[command]
pub async fn dev_upgrade_subscription(
    tier: String,
    app_handle: AppHandle,
) -> Result<SubscriptionStatus, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let service = SubscriptionService::new(pool.clone());
    match identity::resolve_identity(&app_handle, &pool) {
        identity::Identity::Account { user_id, token } => {
            let client = crate::server_client::ServerClient::new();
            let remote = client.dev_upgrade(&token, &tier).await?; // 已登录：失败不降级写本地
            let _ = super::cache_remote_status(&pool, &user_id, &remote);
            let _ = crate::state_sync::StateSync::emit_subscription_changed(&app_handle, &user_id, &remote.tier);
            Ok(SubscriptionStatus { user_id, tier: remote.tier, status: remote.status, expires_at: remote.expires_at })
        }
        identity::Device { machine_id } => {
            let expires_days = if tier == "pro" { Some(30) } else { None };
            let result = service.upgrade_subscription(&machine_id, &tier, expires_days);
            if result.is_ok() {
                let _ = crate::state_sync::StateSync::emit_subscription_changed(&app_handle, &machine_id, &tier);
            }
            result
        }
    }
}

#[command]
pub async fn dev_downgrade_subscription(app_handle: AppHandle) -> Result<SubscriptionStatus, AppError> {
    dev_upgrade_subscription("free".to_string(), app_handle).await
}
```

删除本地 `get_user_id`。

- [ ] **Step 4: 替换其余 5 处 get_user_id**

每处：`use crate::subscription::identity;` 并把 `get_user_id(&app_handle)`（及本地函数定义删除）换成 `identity::resolve_user_id(&app_handle, &pool)`。各文件 pool 变量名以现状为准（没有现成 pool 的就 `app_handle.state::<DbPool>().inner().clone()`）。用 Grep `fn get_user_id` 确认全部清零。

- [ ] **Step 5: 启动时同步**

`src-tauri/src/lib.rs` 的 setup（或 state_sync 初始化处）加一行：`crate::subscription::sync_remote_subscription(&app.handle());`（以 setup 闭包内可拿到 AppHandle 的位置为准）。

- [ ] **Step 6: 测试 + 全量验证**

新增测试：`cache_remote_status` 写入后 `has_feature_access(user_id, "guidebook_distillation") == true`；tier 相同不重复插行。`cargo test --lib` 全绿（1206+）。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/subscription/ src-tauri/src/agents/commands.rs src-tauri/src/pipeline/commands.rs src-tauri/src/book_deconstruction/commands.rs src-tauri/src/guidebook_distillation/commands.rs src-tauri/src/llm/service.rs src-tauri/src/lib.rs
git commit -m "新增(订阅): 身份收口（账号 UUID/machine_id）+ 远程优先与本地缓存降级"
```

---

### Task 5: 前端登录流程打通

**Files:**
- Modify: `src-frontend/src/services/auth.ts`
- Modify: `src-frontend/src/stores/useAuthStore.ts`
- Modify: `src-frontend/src/pages/Login.tsx`（轮询等待 UI + 取消）
- Test: `src-frontend/src/stores/__tests__/useAuthStore.test.ts`（新建或更新现有）
- Modify: `src-tauri/src/mock-tauri.js`（加 oauth_poll_login case，供 vitest 浏览器 mock）

**Interfaces:**
- Consumes: Task 3 命令 `oauth_start { auth_url, dstate }`、`oauth_poll_login -> CurrentSession|null`、`get_current_user -> CurrentSession|null`、`logout(token)`
- Produces:
  - `services/auth.ts`：`OAuthStartResponse { auth_url: string; dstate: string }`（删除 redirect_port 与 oauthCallback）；`oauthStart(provider, invite?)`；`oauthPollLogin(dstate) -> CurrentSession|null`；`getCurrentUser() -> CurrentSession|null`；`CurrentSession { user: UserInfo; token: string }`
  - store：`login(provider, invite?)` 完整流程（打开浏览器 + 2s×60 轮询 + 成功落 user/token）；`cancelLogin()`；`isWaitingForOAuth: boolean`
  - Login.tsx：登录按钮上方加邀请码输入框（占位文案「邀请码（新用户注册必填）」），值随 login 传递；错误 toast 提示「邀请码无效或已使用」场景

- [ ] **Step 1: 写失败测试**

`useAuthStore.test.ts`：mock `@/services/auth` 的 `openOAuthBrowser` 返回 `{ auth_url, dstate: 'd1' }`，`oauthPollLogin` 前两次 null 第三次返回 `{ user: { id: 'u1' }, token: 'tok' }`；断言 `login('google', 'BETA-1')` 完成后 `isLoggedIn === true`、`authToken === 'tok'`、localStorage 有 `sf_auth_token`，且 openOAuthBrowser 收到 `('google', 'BETA-1')`。再一例：`cancelLogin()` 后不再轮询、isWaitingForOAuth=false。

- [ ] **Step 2: services/auth.ts 重写**

```ts
export interface OAuthStartResponse {
  auth_url: string;
  dstate: string;
}

export interface CurrentSession {
  user: UserInfo;
  token: string;
}

export const oauthStart = (provider: string, invite?: string) =>
  loggedInvoke<OAuthStartResponse>('oauth_start', { provider, invite: invite ?? null });

export const oauthPollLogin = (dstate: string) =>
  loggedInvoke<CurrentSession | null>('oauth_poll_login', { dstate });

export const getCurrentUser = () => loggedInvoke<CurrentSession | null>('get_current_user');

export const logout = (token: string) => loggedInvoke<void>('logout', { token });

export const openOAuthBrowser = async (provider: string, invite?: string): Promise<OAuthStartResponse> => {
  const resp = await oauthStart(provider, invite);
  await open(resp.auth_url);
  return resp;
};
```

（删除 `oauthCallback` 与 `redirect_port`。）

- [ ] **Step 3: useAuthStore 重写 login/checkAuth**

```ts
login: async (provider: string, invite?: string) => {
  set({ isLoading: true, isWaitingForOAuth: true });
  try {
    const resp = await openOAuthBrowser(provider, invite);
    const deadline = Date.now() + 120_000;
    while (Date.now() < deadline) {
      if (!get().isWaitingForOAuth) return; // 用户取消
      const session = await oauthPollLogin(resp.dstate);
      if (session) {
        get().setAuthToken(session.token);
        set({ user: session.user, isLoggedIn: true });
        return;
      }
      await new Promise(r => setTimeout(r, 2000));
    }
    throw new Error('登录超时，请重试');
  } finally {
    set({ isLoading: false, isWaitingForOAuth: false });
  }
},

cancelLogin: () => set({ isWaitingForOAuth: false }),

checkAuth: async () => {
  try {
    const session = await getCurrentUser();
    if (session) {
      get().setAuthToken(session.token);
      set({ user: session.user, isLoggedIn: true });
    }
  } catch (e) {
    authLogger.error('Auth check failed', { error: e });
  }
},
```

- [ ] **Step 4: Login.tsx 等待 UI**

`isWaitingForOAuth` 时按钮区替换为「等待浏览器授权…（转圈）+ 取消按钮（调 cancelLogin）」；登录成功 LoginModal 自动关闭（现有 isLoggedIn 逻辑若有）。保持 cinema 主题风格，参考现有文件样式。

- [ ] **Step 5: mock-tauri.js 加 case**

`oauth_poll_login` → 默认返回 `null`（测试可在 mock 层覆盖）；`get_current_user` → `null`。

- [ ] **Step 6: 测试通过**

`npx vitest run src/stores` 全过；`npx tsc --noEmit` 无错；受 oauthCallback 删除影响的文件（Grep `oauthCallback` 全仓前端清零）。

- [ ] **Step 7: Commit**

```bash
git add src-frontend/src/services/auth.ts src-frontend/src/stores/useAuthStore.ts src-frontend/src/pages/Login.tsx src-frontend/src/stores/__tests__/ src-tauri/src/mock-tauri.js
git commit -m "新增(前端): 登录流程打通（浏览器授权 + 轮询换 session + 取消）"
```

---

### Task 6: UpgradeModal 登录引导 + 订阅来源显示

**Files:**
- Modify: `src-frontend/src/components/UpgradeModal.tsx`
- Modify: `src-frontend/src/pages/AccountSettings.tsx`（订阅来源行）
- Test: `src-frontend/src/components/__tests__/UpgradeModal.test.tsx`（新建）

**Interfaces:**
- Consumes: `useAuthStore` 的 `isLoggedIn`、`login(provider)`、`isWaitingForOAuth`；现有 `devUpgradeSubscription`
- Produces: 未登录时弹窗渲染登录引导区（Google/GitHub 按钮 +「暂不登录，仅本设备升级」链接）；已登录保持现状「立即升级」

- [ ] **Step 1: 写失败测试**

用例：未登录 → 显示「登录后升级，Pro 跟随账号」、Google/GitHub 登录按钮与「仅本设备升级」；点「仅本设备升级」→ 调 devUpgradeSubscription('pro')；已登录 → 不显示登录引导，「立即升级」工作（沿用 v0.33.7 链路测试写法，mock `@/services/tauri` 与 `@/stores/useAuthStore`）。

- [ ] **Step 2: 实现 UpgradeModal**

在价格区块上方插入（仅 `!isLoggedIn` 时）：

```tsx
{!isLoggedIn && (
  <div className="mb-5 rounded-lg border border-cinema-700 bg-cinema-800/50 p-3">
    <p className="text-xs text-gray-400 text-center mb-2">
      登录后升级，Pro 跟随账号（换设备不丢）
    </p>
    <div className="flex gap-2">
      <button onClick={() => void login('google')} className="flex-1 py-2 rounded-lg bg-cinema-700 text-sm text-gray-200 hover:bg-cinema-600">
        Google 登录
      </button>
      <button onClick={() => void login('github')} className="flex-1 py-2 rounded-lg bg-cinema-700 text-sm text-gray-200 hover:bg-cinema-600">
        GitHub 登录
      </button>
    </div>
  </div>
)}
```

「立即升级」按钮文案在未登录时改为「暂不登录，仅本设备升级」（同一 handleUpgrade，`isWaitingForOAuth` 时禁用）。

- [ ] **Step 3: AccountSettings 订阅来源**

订阅状态附近加一行：`订阅来源：{isLoggedIn ? '账号同步（跨设备）' : '仅本设备（登录后可同步）'}`。

- [ ] **Step 4: 测试通过 + Commit**

`npx vitest run` 全绿、`tsc` 无错。

```bash
git add src-frontend/src/components/UpgradeModal.tsx src-frontend/src/components/__tests__/ src-frontend/src/pages/AccountSettings.tsx
git commit -m "新增(前端): 升级弹窗登录引导 + 账户页订阅来源显示"
```

---

### Task 7: 部署工作流与运维文档

**Files:**
- Create: `.github/workflows/deploy-server.yml`
- Create: `src-server/.env.example`（若已有则更新：加 `DEV_UPGRADE_ENABLED=true`、`FRONTEND_URL=https://storymoss.top`）
- Modify: `SERVER_DEPLOYMENT.md`（补 nginx 反代与 OAuth App 注册清单）

**Interfaces:**
- Consumes: 现有 `docker-compose.yml`（postgres+server+web）、`SERVER_DEPLOYMENT.md`
- Produces: workflow_dispatch 触发的部署工作流（secrets: `SERVER_SSH_HOST` / `SERVER_SSH_USER` / `SERVER_SSH_KEY`）

- [ ] **Step 1: deploy-server.yml**

```yaml
name: Deploy Server

on:
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Copy server stack to host
        uses: appleboy/scp-action@v0.1.7
        with:
          host: ${{ secrets.SERVER_SSH_HOST }}
          username: ${{ secrets.SERVER_SSH_USER }}
          key: ${{ secrets.SERVER_SSH_KEY }}
          source: "src-server,docker-compose.yml"
          target: "/opt/storymoss/"
          strip_components: 0
      - name: docker compose up
        uses: appleboy/ssh-action@v1.2.0
        with:
          host: ${{ secrets.SERVER_SSH_HOST }}
          username: ${{ secrets.SERVER_SSH_USER }}
          key: ${{ secrets.SERVER_SSH_KEY }}
          script: |
            cd /opt/storymoss
            docker compose up -d --build server
            docker compose ps
```

（web 服务与 storymoss.top 静态站共存方式以主机实际 Web 服务为准，部署文档说明 nginx 反代 `/api/` → `127.0.0.1:8080`；server 容器端口映射改 `127.0.0.1:8080:8080` 避免直接暴露——docker-compose.yml 对应调整。）

- [ ] **Step 2: SERVER_DEPLOYMENT.md 补节**

- Nginx 片段：`location /api/ { proxy_pass http://127.0.0.1:8080; proxy_set_header Host $host; }`
- 手工清单（用户操作）：① Google Cloud Console 建 OAuth client，callback `https://storymoss.top/api/auth/google/callback`；② GitHub Settings → Developer → OAuth Apps，callback `https://storymoss.top/api/auth/github/callback`；③ 主机 `/opt/storymoss/.env` 填 GOOGLE/GITHUB_CLIENT_ID/SECRET、JWT_SECRET（与桌面无关，server 自签自验）、POSTGRES_PASSWORD、FRONTEND_URL=https://storymoss.top；④ GitHub repo secrets 配 SERVER_SSH_*；⑤ 首次 `docker compose up -d`。
- 邀请码发放（Task 2B 配套）：`docker compose exec postgres psql -U storymoss -c "INSERT INTO invite_codes (code, max_uses, note) VALUES ('BETA-XXXX', 1, '发给某某');"`；查余量 `SELECT code, used_count, max_uses FROM invite_codes;`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-server.yml src-server/.env.example SERVER_DEPLOYMENT.md docker-compose.yml
git commit -m "部署(server): deploy-server 工作流 + nginx 反代与 OAuth App 注册清单"
```

---

### Task 8: 版本与文档收尾（v0.34.0）

**Files:**
- Modify: `CHANGELOG.md`（新增 v0.34.0 条目）
- Modify: `AGENTS.md`（头部版本 → v0.34.0）
- Modify: 版本号 5 处（见 Global Constraints）

- [ ] **Step 1: CHANGELOG 条目**

涵盖：server 订阅 API 与桌面 OAuth 中转；桌面登录打通（server 中转轮询）；订阅身份收口 + 远程优先/本地缓存降级 + 双设备同步；前端登录等待 UI、升级弹窗登录引导、订阅来源显示；部署工作流。已知问题沿用条目不动。

- [ ] **Step 2: 版本号 5 处同步为 0.34.0 + AGENTS.md 头部**

- [ ] **Step 3: 全量验证**

`cd src-tauri && cargo test --lib` 全绿；`cd src-frontend && npx vitest run && npx tsc --noEmit` 全绿；`cd src-server && DATABASE_URL=... cargo test` 全绿；`cargo +nightly fmt`；prettier。

- [ ] **Step 4: Commit（不推送、不打 tag，等用户确认发布）**

```bash
git add CHANGELOG.md AGENTS.md src-tauri/Cargo.toml src-tauri/tauri.conf.json src-frontend/package.json landing/src/hooks/useLatestRelease.ts Cargo.lock
git commit -m "发布: v0.34.0 OAuth 登录绑定订阅 + 跨设备同步"
```

- [ ] **Step 5: 手动验证清单（告知用户，需 server 已部署 + OAuth App 已注册）**

1. 双设备/双系统用户目录：设备 A 登录 Google → 升级 Pro → 设备 B 登录同账号 → 启动后 Pro 自动生效
2. 未登录升级 Pro → 提示仅本设备；登录后……（当前设计：本设备 Pro 不自动迁移到账号，CHANGELOG 注明）
3. 断网启动：已登录用户离线降级用缓存 tier，写作流不中断
4. 退出登录 → 订阅回落 machine_id 设备身份

---

## Self-Review 记录

- 规格覆盖：设计文档 ①②③④⑤⑥ 节 ↔ Task 3+5 / 1+2 / 4 / 5+6 / 7 / 各 Task 测试节 + Task 8 清单，无缺口
- 类型一致性：`SubscriptionResponse`/`RemoteSubscription`/`CurrentSession`/`Identity` 在 Task 1/3/4/5 间签名一致；`sync_remote_subscription` 在 Task 3 占位、Task 4 实装已注明
- 已知风险：① 本地 V033 sessions/users 表 DDL 以实际为准微调 Task 3 SQL；② sqlx 编译期需 DATABASE_URL；③ 本设备 Pro 不自动迁移到账号（设计取舍，付款接入时再议）
