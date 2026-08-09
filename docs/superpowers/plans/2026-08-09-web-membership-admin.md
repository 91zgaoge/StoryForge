# 网站侧会员系统实现计划（Admin 后台 + 邀请码发放 + JWT 吊销 + expires_at 生效）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 网站侧补全会员系统：管理员角色与登录管理、邀请码发放后台、用户 Dashboard 订阅状态；同车修终审 backlog 的 JWT 吊销（I3）与 expires_at 生效（I2/#2/#12）。

**Architecture:** 扩展 src-server（migration 004 + Admin API + auth 强化）与 src-server-web（/admin 三页签 + Dashboard 卡片）；桌面端仅 cache_remote_status 透传 expires_at 小改。本期并入未发布的 v0.34.0（不改版本号）。

**Tech Stack:** Actix-web 4 / sqlx 0.8(Postgres) / jsonwebtoken 9；React 18 + react-router 7 + axios + zustand + tailwind（src-server-web）；vitest（web 新建最小基建）。

**设计文档：** `docs/plans/2026-08-09-web-membership-admin-design.md`（已批准）

## Global Constraints

- 中文 conventional commit；提交不夹带 `.recovery/`；**只提交不推送不打 tag**（等用户统一发布 v0.34.0）
- pre-commit 钩子：`cargo +nightly fmt` + prettier 必须通过
- server 编译/测试需 DATABASE_URL：`pg_isready -h localhost -p 5432` 检查，不在则参照 `.superpowers/sdd/task-1-report.md` 重建便携 postgres；`src-server/.env` 已有 DATABASE_URL
- 账本 `.superpowers/sdd/progress.md` 每个 Task 完成后追加一行
- Admin API 全部写操作 `log::info!` 审计（操作者/动作/对象）
- require_admin 每次查库校验 role，不信任 JWT claim
- 邀请码作废为软删（revoked_at）；作废码注册必须被拒
- 版本号不动（并入 v0.34.0，CHANGELOG 在既有 v0.34.0 条目上增补）

---

### Task 1: migration 004 + JWT 吊销/禁用 + /auth/me 带 role

**Files:**
- Create: `src-server/migrations/004_admin_and_invite_grant.sql`
- Modify: `src-server/src/auth/jwt.rs`（from_request 加 session/禁用校验）
- Modify: `src-server/src/auth/mod.rs`（UserResponse 加 role）
- Modify: `src-server/src/auth/handlers.rs`（get_me 与 callback 的 UserResponse 查询带 role）

**Interfaces:**
- Consumes: 现有 `AuthClaims` FromRequest、`sessions` 表（token 列存 JWT 全文）
- Produces（后续任务依赖）:
  - `pub async fn authenticate(req: &HttpRequest) -> Result<AuthClaims, actix_web::Error>`（jwt.rs，Task 4 的 AdminUser 提取器复用）
  - `UserResponse { id, email, display_name, avatar_url, role }`（role: String）
  - users 表新列 `role TEXT NOT NULL DEFAULT 'user'`、`disabled_at TIMESTAMPTZ`；invite_codes 新列 `grant_pro_days INT`、`created_by UUID`、`revoked_at TIMESTAMPTZ`

- [ ] **Step 1: 迁移 004**

```sql
-- 会员系统：管理员角色 + 禁用 + 邀请码赠 Pro/作废

ALTER TABLE users ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'user';
ALTER TABLE users ADD COLUMN IF NOT EXISTS disabled_at TIMESTAMPTZ;

ALTER TABLE invite_codes ADD COLUMN IF NOT EXISTS grant_pro_days INT;
ALTER TABLE invite_codes ADD COLUMN IF NOT EXISTS created_by UUID;
ALTER TABLE invite_codes ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;
```

- [ ] **Step 2: 写失败测试**

jwt.rs 测试模块（文件无测试模块则新建）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::Payload, test::TestRequest, web, FromRequest};

    async fn seed_user_session(pool: &sqlx::PgPool, token: &str) -> uuid::Uuid {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", uid, "jwt@t.com")
            .execute(pool).await.unwrap();
        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
            uid, token
        )
        .execute(pool).await.unwrap();
        uid
    }

    fn req_with(pool: &sqlx::PgPool, token: &str) -> actix_web::HttpRequest {
        TestRequest::default()
            .app_data(web::Data::new(pool.clone()))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_http_request()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn valid_token_with_active_session_passes(pool: sqlx::PgPool) {
        let uid = seed_user_session(&pool, "tok-valid").await;
        // create_token 签名用 CONFIG.jwt_secret；测试直接造真实 JWT
        let token = create_token(&uid.to_string()).unwrap();
        sqlx::query!("UPDATE sessions SET token = $1 WHERE user_id = $2", token, uid)
            .execute(&pool).await.unwrap();
        let req = req_with(&pool, &token);
        let claims = AuthClaims::from_request(&req, &mut Payload::None).await.unwrap();
        assert_eq!(claims.sub, uid.to_string());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn revoked_session_is_rejected(pool: sqlx::PgPool) {
        let uid = seed_user_session(&pool, "tok-x").await;
        let token = create_token(&uid.to_string()).unwrap();
        // 不把 token 写入 sessions（= 已注销/从未登记）
        let req = req_with(&pool, &token);
        assert!(AuthClaims::from_request(&req, &mut Payload::None).await.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn disabled_user_is_rejected(pool: sqlx::PgPool) {
        let uid = seed_user_session(&pool, "tok-y").await;
        let token = create_token(&uid.to_string()).unwrap();
        sqlx::query!("UPDATE sessions SET token = $1 WHERE user_id = $2", token, uid)
            .execute(&pool).await.unwrap();
        sqlx::query!("UPDATE users SET disabled_at = NOW() WHERE id = $1", uid)
            .execute(&pool).await.unwrap();
        let req = req_with(&pool, &token);
        assert!(AuthClaims::from_request(&req, &mut Payload::None).await.is_err());
    }
}
```

运行 `cd src-server && cargo test auth::jwt` — 预期 FAIL（当前只验签不查库，后两例会通过=断言失败）。

- [ ] **Step 3: 实现**

jwt.rs 的 `AuthClaims::from_request` 重写为委托新 pub 函数：

```rust
/// 完整鉴权：验签 + session 未吊销 + 用户未禁用（AdminUser 提取器同用）
pub async fn authenticate(req: &HttpRequest) -> Result<AuthClaims, actix_web::Error> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| error::ErrorUnauthorized("Missing Authorization header"))?;

    let claims = validate_token(token).map_err(|_| error::ErrorUnauthorized("Invalid token"))?;

    let pool = req
        .app_data::<web::Data<sqlx::PgPool>>()
        .cloned()
        .ok_or_else(|| error::ErrorInternalServerError("missing db pool"))?;

    let user_uuid = claims.sub.parse::<uuid::Uuid>().unwrap_or_default();
    let row = sqlx::query!(
        "SELECT u.disabled_at FROM sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.token = $1 AND s.expires_at > NOW() AND s.user_id = $2",
        token,
        user_uuid
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("auth db check failed: {}", e);
        error::ErrorInternalServerError("db error")
    })?;

    match row {
        Some(r) if r.disabled_at.is_none() => Ok(claims),
        _ => Err(error::ErrorUnauthorized("Session revoked or user disabled")),
    }
}

impl FromRequest for AuthClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { authenticate(&req).await })
    }
}
```

`auth/mod.rs` 的 `UserResponse` 加 `pub role: String`；handlers.rs 两处构建 UserResponse 的 SQL（callback 与 get_me）SELECT 加 `role`、结构体初始化加 `role: row.role`。

- [ ] **Step 4: 测试通过**

`cargo test auth::jwt` 3 例全过；`cargo test` 全量不回归（基线 10）；`cargo +nightly fmt`。

- [ ] **Step 5: Commit**

```bash
git add src-server/migrations/004_admin_and_invite_grant.sql src-server/src/auth/
git commit -m "新增(server): 管理员/禁用列 + JWT 吊销校验（session+disabled 查库）+ /auth/me 带 role"
```

---

### Task 2: expires_at 生效（/me 过期降级 + upsert_tier 读回 + days 参数）

**Files:**
- Modify: `src-server/src/api/subscription.rs`

**Interfaces:**
- Consumes: Task 1 的 004 表结构
- Produces:
  - `pub(crate) async fn upsert_tier(pool: &PgPool, user_id: Uuid, tier: &str, pro_days: Option<i64>) -> Result<SubscriptionRow, sqlx::Error>`——Tier free→expires NULL；pro/enterprise 给 days（None 时 pro 默认 30）；**写后读回真实行**（Task 4 admin 调订阅复用）
  - `GET /subscription/me`：非 free 且 `expires_at < now()` → 懒更新库为 free 并按 free 返回

- [ ] **Step 1: 写失败测试**

subscription.rs 测试模块加：

```rust
#[sqlx::test(migrations = "./migrations")]
async fn expired_pro_is_downgraded_lazily(pool: PgPool) {
    let uid = uuid::Uuid::new_v4();
    sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", uid, "e@t.com")
        .execute(&pool).await.unwrap();
    sqlx::query!(
        "INSERT INTO subscriptions (user_id, tier, expires_at) VALUES ($1, 'pro', NOW() - INTERVAL '1 day')",
        uid
    )
    .execute(&pool).await.unwrap();

    let row = get_or_create(&pool, uid).await.unwrap();
    assert_eq!(row.tier, "free");
    assert_eq!(row.expires_at, None);
    // 库里也懒更新为 free
    let tier: String = sqlx::query_scalar!("SELECT tier FROM subscriptions WHERE user_id = $1", uid)
        .fetch_one(&pool).await.unwrap();
    assert_eq!(tier, "free");
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_tier_with_days_and_reads_back(pool: PgPool) {
    let uid = uuid::Uuid::new_v4();
    sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", uid, "d@t.com")
        .execute(&pool).await.unwrap();

    let row = upsert_tier(&pool, uid, "pro", Some(90)).await.unwrap();
    assert_eq!(row.tier, "pro");
    let exp = row.expires_at.unwrap();
    assert!(exp > chrono::Utc::now() + chrono::Duration::days(80));

    // 改回 free 读回真实行
    let row = upsert_tier(&pool, uid, "free", None).await.unwrap();
    assert_eq!(row.tier, "free");
    assert_eq!(row.expires_at, None);
}
```

运行 `cargo test api::subscription` — FAIL（现 upsert_tier 无 days 参数且不读回）。

- [ ] **Step 2: 实现**

- `upsert_tier` 改签名加 `pro_days: Option<i64>`，可见性 `pub(crate)`；expires 计算：`"free" → None`；`Some(d) → now+d`；`("pro", None) → now+30`；其余 None。写后用 `SELECT tier, status, expires_at ... fetch_one` 读回构造 SubscriptionRow
- `get_or_create` 的 existing 分支：tier != "free" 且 expires_at < now → `UPDATE subscriptions SET tier='free', expires_at=NULL, updated_at=NOW()` 后返回 free 行
- `dev_upgrade` handler 调用点改 `upsert_tier(pool.get_ref(), user_id, tier, None)`

- [ ] **Step 3: 测试通过 + Commit**

`cargo test api::subscription` 全过 + 全量不回归 + fmt。

```bash
git add src-server/src/api/subscription.rs
git commit -m "新增(server): 订阅 expires_at 生效（/me 过期懒降级 + upsert_tier days 参数并读回）"
```

---

### Task 3: 邀请码赠 Pro 注册联动 + 作废码拒绝

**Files:**
- Modify: `src-server/src/auth/handlers.rs`（find_or_create_user_gated）

**Interfaces:**
- Consumes: Task 1 的 `invite_codes.grant_pro_days/revoked_at`；Task 2 的订阅写入语义
- Produces: 占码 UPDATE 加 `AND revoked_at IS NULL` 且 `RETURNING grant_pro_days`；注册事务内码带 grant_pro_days 时 `INSERT INTO subscriptions (user_id, tier, expires_at, source) VALUES ($1, 'pro', $2, 'invite') ON CONFLICT (user_id) DO NOTHING`

- [ ] **Step 1: 写失败测试**

handlers.rs 测试模块加：

```rust
#[sqlx::test(migrations = "./migrations")]
async fn invite_with_grant_pro_days_creates_pro_subscription(pool: PgPool) {
    sqlx::query!("INSERT INTO invite_codes (code, grant_pro_days) VALUES ('VIP-1', 90)")
        .execute(&pool).await.unwrap();
    let profile = test_profile("vip@t.com", "github", "gh-vip");
    let uid = find_or_create_user_gated(&pool, &profile, Some("VIP-1".into())).await.unwrap();

    let (tier, exp): (String, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as!(
        "SELECT tier, expires_at FROM subscriptions WHERE user_id = $1", uid
    )
    .fetch_one(&pool).await.unwrap();
    assert_eq!(tier, "pro");
    assert!(exp.unwrap() > chrono::Utc::now() + chrono::Duration::days(80));
}

#[sqlx::test(migrations = "./migrations")]
async fn revoked_invite_is_rejected(pool: PgPool) {
    sqlx::query!("INSERT INTO invite_codes (code, revoked_at) VALUES ('DEAD-1', NOW())")
        .execute(&pool).await.unwrap();
    let profile = test_profile("dead@t.com", "github", "gh-dead");
    assert!(find_or_create_user_gated(&pool, &profile, Some("DEAD-1".into())).await.is_err());
}
```

运行 `cargo test auth::handlers` — FAIL（无 RETURNING/作废条件）。

- [ ] **Step 2: 实现**

占码段改为：

```rust
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
```

oauth 关联 INSERT 之后、`tx.commit()` 之前加：

```rust
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
```

- [ ] **Step 3: 测试通过 + Commit**

`cargo test auth::handlers` 全过（基线 6 + 新 2）+ fmt。

```bash
git add src-server/src/auth/handlers.rs
git commit -m "新增(server): 邀请码赠 Pro 注册联动 + 作废码拒绝（revoked_at 软删生效）"
```

---

### Task 4: Admin API（用户管理 + 订阅代调 + 邀请码 CRUD）

**Files:**
- Create: `src-server/src/api/admin.rs`
- Modify: `src-server/src/api/mod.rs`（`mod admin;` + `.configure(admin::init_routes)`）
- Modify: `src-server/src/api/subscription.rs`（upsert_tier 已 pub(crate)，Task 2 完成）

**Interfaces:**
- Consumes: Task 1 `authenticate(req)`；Task 2 `upsert_tier(pool, uid, tier, days)`
- Produces（Task 7/8 前端按此契约）:
  - `AdminUser { user_id: Uuid }` FromRequest 提取器：authenticate 后查 users.role，非 admin → 403
  - `GET /api/admin/users?q=` → `[{ id, email, display_name, role, tier, expires_at, disabled_at, created_at }]`
  - `POST /api/admin/users/{id}/role` body `{"role":"admin"|"user"}`；不能降级自己（400）
  - `POST /api/admin/users/{id}/disable`（disabled_at=NOW + 删其 sessions）/ `POST /api/admin/users/{id}/enable`
  - `POST /api/admin/users/{id}/subscription` body `{"tier":"pro"|"free"|"enterprise","days":90|null}`
  - `GET /api/admin/invite-codes` → `[{ code, max_uses, used_count, grant_pro_days, note, created_at, revoked_at }]`
  - `POST /api/admin/invite-codes` body `{"count":1..=100,"max_uses":1,"grant_pro_days":90|null,"note":"..."|null}` → 返回生成的码列表；码格式 `SM-` + 8 位大写 hex（uuid v4 simple 前 8 位）
  - `POST /api/admin/invite-codes/{code}/revoke`；已作废或不存在 → 404

- [ ] **Step 1: 写失败测试（HTTP 层，actix test service）**

admin.rs 测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    async fn seed_user(pool: &PgPool, email: &str, role: &str) -> (uuid::Uuid, String) {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!("INSERT INTO users (id, email, role) VALUES ($1, $2, $3)", uid, email, role)
            .execute(pool).await.unwrap();
        let token = crate::auth::jwt::create_token(&uid.to_string()).unwrap();
        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
            uid, token
        )
        .execute(pool).await.unwrap();
        (uid, token)
    }

    macro_rules! admin_app {
        ($pool:expr) => {
            test::init_service(
                App::new()
                    .app_data(web::Data::new($pool.clone()))
                    .service(web::scope("/api/admin").configure(init_routes)),
            )
            .await
        };
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn non_admin_gets_403_admin_gets_200(pool: PgPool) {
        let app = admin_app!(pool);
        // 无 token → 401
        let req = test::TestRequest::get().uri("/api/admin/users").to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 401);
        // 普通用户 → 403
        let (_, tok) = seed_user(&pool, "u@t.com", "user").await;
        let req = test::TestRequest::get().uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", tok))).to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 403);
        // admin → 200
        let (_, tok) = seed_user(&pool, "a@t.com", "admin").await;
        let req = test::TestRequest::get().uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", tok))).to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 200);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn disable_user_deletes_sessions_and_blocks_login(pool: PgPool) {
        let app = admin_app!(pool);
        let (admin_id, admin_tok) = seed_user(&pool, "a@t.com", "admin").await;
        let (victim, victim_tok) = seed_user(&pool, "v@t.com", "user").await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/admin/users/{}/disable", victim))
            .insert_header(("Authorization", format!("Bearer {}", admin_tok)))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 200);

        // sessions 已删 → 受害者 token 立即失效
        let req = test::TestRequest::get().uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", victim_tok))).to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 401);
        let _ = admin_id;
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn cannot_demote_self(pool: PgPool) {
        let app = admin_app!(pool);
        let (admin_id, admin_tok) = seed_user(&pool, "a@t.com", "admin").await;
        let req = test::TestRequest::post()
            .uri(&format!("/api/admin/users/{}/role", admin_id))
            .insert_header(("Authorization", format!("Bearer {}", admin_tok)))
            .set_json(serde_json::json!({"role": "user"}))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 400);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_and_revoke_invite_codes(pool: PgPool) {
        let app = admin_app!(pool);
        let (_, admin_tok) = seed_user(&pool, "a@t.com", "admin").await;

        let req = test::TestRequest::post().uri("/api/admin/invite-codes")
            .insert_header(("Authorization", format!("Bearer {}", admin_tok)))
            .set_json(serde_json::json!({"count": 3, "max_uses": 1, "grant_pro_days": 30, "note": "测试批"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        let codes = body["codes"].as_array().unwrap();
        assert_eq!(codes.len(), 3);
        assert!(codes[0].as_str().unwrap().starts_with("SM-"));

        // 作废其一 → 列表里 revoked_at 非空；再作废 → 404
        let code = codes[0].as_str().unwrap();
        let req = test::TestRequest::post()
            .uri(&format!("/api/admin/invite-codes/{}/revoke", code))
            .insert_header(("Authorization", format!("Bearer {}", admin_tok)))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 200);
        let req = test::TestRequest::post()
            .uri(&format!("/api/admin/invite-codes/{}/revoke", code))
            .insert_header(("Authorization", format!("Bearer {}", admin_tok)))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 404);
    }
}
```

运行 `cargo test api::admin` — FAIL（模块不存在）。

- [ ] **Step 2: 实现 admin.rs**

骨架（各 handler 按 Interfaces 契约实现；以下为关键片段，完整实现跟随既有 subscription.rs 风格）：

```rust
//! Admin API — 会员系统管理后台（require_admin 查库校验）

use actix_web::{dev::Payload, error, get, post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;

use crate::auth::jwt::authenticate;

pub struct AdminUser {
    pub user_id: uuid::Uuid,
}

impl FromRequest for AdminUser {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let claims = authenticate(&req).await?;
            let pool = req
                .app_data::<web::Data<PgPool>>()
                .cloned()
                .ok_or_else(|| error::ErrorInternalServerError("missing db pool"))?;
            let user_id = claims.sub.parse::<uuid::Uuid>().unwrap_or_default();
            let role: String = sqlx::query_scalar!("SELECT role FROM users WHERE id = $1", user_id)
                .fetch_one(pool.get_ref())
                .await
                .map_err(|e| {
                    log::error!("admin role check failed: {}", e);
                    error::ErrorInternalServerError("db error")
                })?;
            if role != "admin" {
                return Err(error::ErrorForbidden("Admin required"));
            }
            Ok(AdminUser { user_id })
        })
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users)
        .service(set_user_role)
        .service(disable_user)
        .service(enable_user)
        .service(set_user_subscription)
        .service(list_invite_codes)
        .service(create_invite_codes)
        .service(revoke_invite_code);
}
```

要点：
- `list_users`：`SELECT u.id, u.email, u.display_name, u.role, u.disabled_at, u.created_at, s.tier, s.expires_at AS sub_expires_at FROM users u LEFT JOIN subscriptions s ON s.user_id = u.id WHERE ($1::text IS NULL OR u.email ILIKE '%'||$1||'%' OR u.display_name ILIKE '%'||$1||'%') ORDER BY u.created_at DESC LIMIT 200`，q 空串传 None
- `set_user_role`：body role ∈ {admin, user}；`id == admin.user_id && role != "admin"` → 400 `{"error":"cannot demote self"}`；审计 `log::info!("[admin] {} set role of {} to {}", admin.user_id, id, role)`
- `disable_user`：`UPDATE users SET disabled_at = NOW() WHERE id = $1` + `DELETE FROM sessions WHERE user_id = $1`（同事务）；enable 反向
- `set_user_subscription`：tier 白名单校验后调 `crate::api::subscription::upsert_tier(pool.get_ref(), id, &tier, body.days).await`（days 由 body Option<i64> 直接传）
- `create_invite_codes`：count 1..=100 校验；循环 `format!("SM-{}", &uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase())` INSERT（撞 PK 重试一次）；返回 `{"codes":[...]}`；审计
- `revoke_invite_code`：`UPDATE invite_codes SET revoked_at = NOW() WHERE code = $1 AND revoked_at IS NULL`，0 行 → 404

`api/mod.rs`：`mod admin;`（admin 需在 crate 内可见：`pub(crate) mod admin;`）+ scope 链加 `.configure(admin::init_routes)`——注意 admin 路由自带 `/admin` 前缀：在 `web::scope("/api")` 内用 `web::scope("/admin").configure(admin::init_routes)` 嵌套，handler 路径写 `/users`、`/invite-codes` 等（与测试的 `/api/admin/users` 对齐）。

- [ ] **Step 3: 测试通过 + Commit**

`cargo test api::admin` 4 例全过 + 全量不回归 + fmt。

```bash
git add src-server/src/api/admin.rs src-server/src/api/mod.rs
git commit -m "新增(server): Admin API（用户/订阅代调/邀请码 CRUD，require_admin 查库+审计日志）"
```

---

### Task 5: 桌面端 cache_remote_status 透传 expires_at

**Files:**
- Modify: `src-tauri/src/subscription/mod.rs`（cache_remote_status）

**Interfaces:**
- Consumes: server 端 expires_at 已真实生效（Task 2/3）
- Produces: `cache_remote_status` 按 `remote.expires_at`（RFC3339）换算天数写入本地缓存，不再恒写 30 天

- [ ] **Step 1: 写失败测试**

mod.rs 测试模块加：

```rust
#[test]
fn cache_remote_status_uses_remote_expiry_days() {
    let pool = crate::db::connection::create_test_pool().unwrap();
    let remote = crate::server_client::RemoteSubscription {
        tier: "pro".into(),
        status: "active".into(),
        expires_at: Some((chrono::Local::now() + chrono::Duration::days(90)).to_rfc3339()),
    };
    cache_remote_status(&pool, "u-exp", &remote).unwrap();

    // 本地缓存行的 expires_at 应≈90 天后（容差 1 天），而非恒 30 天
    let svc = SubscriptionService::new(pool.clone());
    let status = svc.get_or_create_subscription("u-exp").unwrap();
    let exp = status.expires_at.unwrap();
    let exp_dt = chrono::DateTime::parse_from_rfc3339(&exp).unwrap();
    assert!(exp_dt > chrono::Local::now() + chrono::Duration::days(80));
}
```

运行 `cd src-tauri && cargo test subscription::tests::cache` — FAIL（现实现恒 30 天）。

- [ ] **Step 2: 实现**

`cache_remote_status` 的 tier 不同分支：

```rust
if current.tier != remote.tier {
    let expires_days = remote
        .expires_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|exp| {
            let secs = exp.timestamp() - chrono::Local::now().timestamp();
            (secs / 86400).max(0) as i32
        });
    let expires_days = match remote.tier.as_str() {
        "free" => None,
        _ => expires_days.or(Some(30)), // 无 expires 的 pro（dev 通道）兜底 30 天
    };
    service.upgrade_subscription(user_id, &remote.tier, expires_days)?;
}
```

- [ ] **Step 3: 测试通过 + Commit**

`cargo test subscription` 全过 + `cargo test --lib` 不回归（基线 1216）+ fmt。

```bash
git add src-tauri/src/subscription/mod.rs
git commit -m "修复(订阅): 本地缓存透传 server expires_at（不再恒写 30 天）"
```

---

### Task 6: Web 测试基建 + admin.ts + 路由守卫 + Dashboard 订阅卡片

**Files:**
- Modify: `src-server-web/package.json`（devDeps: vitest, jsdom, @testing-library/react, @testing-library/jest-dom, @testing-library/user-event；script `test: vitest run`）
- Create: `src-server-web/vitest.config.ts`、`src-server-web/src/test/setup.ts`
- Create: `src-server-web/src/api/client.ts`（axios 实例：baseURL `/api`、拦截器注入 `sf_token`、401 清 token 跳 /login）
- Create: `src-server-web/src/api/admin.ts`
- Create: `src-server-web/src/api/subscription.ts`
- Create: `src-server-web/src/pages/admin/AdminLayout.tsx`（页签导航 + 守卫）
- Modify: `src-server-web/src/App.tsx`（/admin 路由）
- Modify: `src-server-web/src/pages/DashboardPage.tsx`（订阅卡片 + admin 入口）
- Test: `src-server-web/src/pages/admin/__tests__/AdminLayout.test.tsx`、`src-server-web/src/pages/__tests__/DashboardPage.test.tsx`

**Interfaces:**
- Consumes: Task 4 Admin API；Task 1 `/auth/me` 带 role；`GET /api/subscription/me`
- Produces:
  - `api/client.ts` 默认导出 axios 实例（后续页面统一使用）
  - `admin.ts`：`listUsers(q?)`、`setUserRole(id, role)`、`disableUser(id)`、`enableUser(id)`、`setUserSubscription(id, tier, days?)`、`listInviteCodes()`、`createInviteCodes({count,max_uses,grant_pro_days?,note?})`、`revokeInviteCode(code)`
  - `subscription.ts`：`getMySubscription() -> { tier, status, expires_at }`
  - `AdminLayout`：读 `sf_role`（登录时 /auth/me 存 localStorage；Dashboard 也存）≠ 'admin' → `<Navigate to="/dashboard" replace />`；否则渲染页签导航（邀请码/用户/管理员）+ `<Outlet />`
  - `getRole()` 辅助（读 localStorage sf_role）

- [ ] **Step 1: 测试基建**

`npm i -D vitest jsdom @testing-library/react @testing-library/jest-dom @testing-library/user-event`；vitest.config.ts：

```ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: { environment: 'jsdom', setupFiles: './src/test/setup.ts', globals: true },
})
```

`src/test/setup.ts`：`import '@testing-library/jest-dom'`。package.json scripts 加 `"test": "vitest run"`。

- [ ] **Step 2: 写失败测试**

`AdminLayout.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import AdminLayout from '../AdminLayout'

describe('AdminLayout 守卫', () => {
  it('非 admin 跳回 /dashboard', () => {
    localStorage.setItem('sf_role', 'user')
    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/admin" element={<AdminLayout />}>
            <Route index element={<div>邀请码页</div>} />
          </Route>
          <Route path="/dashboard" element={<div>Dashboard 页</div>} />
        </Routes>
      </MemoryRouter>
    )
    expect(screen.getByText('Dashboard 页')).toBeInTheDocument()
    expect(screen.queryByText('邀请码页')).not.toBeInTheDocument()
  })

  it('admin 看到页签导航', () => {
    localStorage.setItem('sf_role', 'admin')
    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/admin" element={<AdminLayout />}>
            <Route index element={<div>邀请码页</div>} />
          </Route>
        </Routes>
      </MemoryRouter>
    )
    expect(screen.getByText('邀请码')).toBeInTheDocument()
    expect(screen.getByText('用户')).toBeInTheDocument()
    expect(screen.getByText('管理员')).toBeInTheDocument()
  })
})
```

`DashboardPage.test.tsx`：mock `../api/subscription` 的 `getMySubscription` 返回 `{ tier: 'pro', status: 'active', expires_at: '2026-09-08T00:00:00Z' }`，mock axios（auth/me 返回 user+role），断言渲染「专业版」字样与到期日期；free 用户断言显示升级引导文案。运行 `npm test` — FAIL。

- [ ] **Step 3: 实现**

- `api/client.ts`：

```ts
import axios from 'axios'

const client = axios.create({ baseURL: import.meta.env.VITE_API_URL || '/api' })

client.interceptors.request.use(config => {
  const token = localStorage.getItem('sf_token')
  if (token) config.headers.Authorization = `Bearer ${token}`
  return config
})

client.interceptors.response.use(
  resp => resp,
  err => {
    if (err.response?.status === 401) {
      localStorage.removeItem('sf_token')
      localStorage.removeItem('sf_role')
      window.location.href = '/login'
    }
    return Promise.reject(err)
  }
)

export default client
```

- `api/admin.ts` / `api/subscription.ts`：基于 client 的薄封装，返回 `res.data`
- `AdminLayout.tsx`：守卫 + tailwind 页签（NavLink，cinema 主题跟随 DashboardPage 现有类名风格），子路由 `<Outlet />`
- `App.tsx` 加：

```tsx
<Route path="/admin" element={<AdminLayout />}>
  <Route index element={<InviteCodesPage />} />
  <Route path="users" element={<UsersPage />} />
  <Route path="admins" element={<AdminsPage />} />
</Route>
```

（三个页面组件由 Task 7/8 创建，本任务先建占位组件文件 `src/pages/admin/InviteCodesPage.tsx` 等，内容为带标题的空页，Task 7/8 填充——占位必须真实渲染标题文本。）
- `DashboardPage.tsx`：登录时把 `/auth/me` 的 `role` 存 `localStorage.sf_role`；新增「订阅状态」卡片（调 `getMySubscription`：pro → 「专业版 · 有效期至 YYYY-MM-DD」；free → 「免费版 + 下载 StoryMoss 客户端升级 Pro」）；`sf_role === 'admin'` 时顶部加「管理后台」Link

- [ ] **Step 4: 测试通过 + Commit**

`npm test` 全过 + `npm run build`（tsc + vite）无错 + prettier 改动文件。

```bash
git add src-server-web/
git commit -m "新增(web): 测试基建 + admin 路由守卫 + Dashboard 订阅卡片与管理后台入口"
```

---

### Task 7: Web 邀请码管理页

**Files:**
- Modify: `src-server-web/src/pages/admin/InviteCodesPage.tsx`（占位 → 完整实现）
- Test: `src-server-web/src/pages/admin/__tests__/InviteCodesPage.test.tsx`

**Interfaces:**
- Consumes: Task 6 `admin.ts` 的 `listInviteCodes/createInviteCodes/revokeInviteCode`
- Produces: 邀请码管理完整页（生成表单 + 列表 + 作废 + 复制）

- [ ] **Step 1: 写失败测试**

mock `@/api/admin`（vite alias 若无则用相对路径 `../../../api/admin`，以 vitest.config 的 resolve.alias 配置为准——若无 alias，在 vitest.config.ts 加 `resolve: { alias: { '@': '/src' } }` 与 vite 主配置对齐，先查 src-server-web/vite.config 是否已有）：

- 渲染列表：mock listInviteCodes 返回 2 行（含 revoked 一行），断言码、用量 `1/3`、赠 Pro 天数、「已作废」标记
- 生成：填数量 5 / 赠 Pro 30 → 点「生成」→ 断言 createInviteCodes 收到 `{count:5, max_uses:1, grant_pro_days:30, note:...}` 且新码出现在列表
- 作废：点「作废」→ 确认后调 revokeInviteCode

运行 `npm test` — FAIL。

- [ ] **Step 2: 实现**

页面结构（cinema 主题、tailwind，跟随 DashboardPage 风格）：

- 顶部生成表单卡片：数量（number 1-100，默认 10）、每码可用次数（默认 1）、赠 Pro 天数（select：不赠/7/30/90/365 → 0 视为不赠传 null）、备注（text）+「生成邀请码」按钮；生成成功 toast 并展示新码（一键复制按钮用 `navigator.clipboard.writeText`）
- 列表表格：码（等宽字体）/ 用量 used/max / 赠 Pro / 备注 / 状态（生效中=灰绿、已作废=红）/ 创建时间 / 操作（生效中显示「作废」，二次确认 `window.confirm`）

- [ ] **Step 3: 测试通过 + Commit**

`npm test` + `npm run build` + prettier。

```bash
git add src-server-web/src/pages/admin/
git commit -m "新增(web): 邀请码管理页（批量生成/赠 Pro 天数/作废/复制）"
```

---

### Task 8: Web 用户管理页 + 管理员页签

**Files:**
- Modify: `src-server-web/src/pages/admin/UsersPage.tsx`（占位 → 完整实现）
- Modify: `src-server-web/src/pages/admin/AdminsPage.tsx`（占位 → 完整实现）
- Test: `src-server-web/src/pages/admin/__tests__/UsersPage.test.tsx`

**Interfaces:**
- Consumes: Task 6 `admin.ts` 全部用户/角色 API
- Produces: 用户管理页（搜索/表格/赠 Pro/禁用）+ 管理员页签（列表/提拔/降级）

- [ ] **Step 1: 写失败测试**

UsersPage：mock listUsers 返回 admin+user 两行；断言渲染邮箱/tier/角色/状态；点「赠 Pro 30 天」调 setUserSubscription(id, 'pro', 30)；点「禁用」二次确认后调 disableUser；搜索框输入触发 listUsers(q)。AdminsPage 可不单测（复用 UsersPage 的 role 操作，列表过滤 role==='admin'）。

运行 `npm test` — FAIL。

- [ ] **Step 2: 实现**

- `UsersPage.tsx`：搜索框（防抖 300ms 或直接回车/按钮触发，选简单：输入即调）+ 表格（邮箱/昵称/tier/角色/状态：正常·禁用/注册时间）+ 行操作下拉或按钮组：「赠 Pro 30 天」「赠 Pro 90 天」「改为免费」「禁用/启用」；操作成功后刷新列表；禁用二次确认
- `AdminsPage.tsx`：复用 listUsers 过滤 `role === 'admin'` 展示；「提拔管理员」输入邮箱调 listUsers(q) 找到后 setUserRole(id, 'admin')；降级按钮（列表中标记「我」的行不显示降级）

- [ ] **Step 3: 测试通过 + Commit**

`npm test` + `npm run build` + prettier。

```bash
git add src-server-web/src/pages/admin/
git commit -m "新增(web): 用户管理页（搜索/赠 Pro/禁用）+ 管理员页签（提拔/降级）"
```

---

### Task 9: 文档收尾 + 全量验证

**Files:**
- Modify: `CHANGELOG.md`（v0.34.0 条目增补本期内容）
- Modify: `SERVER_DEPLOYMENT.md`（加「指定首个管理员」节）
- Modify: `docs/plans/2026-08-09-web-membership-admin-design.md`（如实现有偏差则回写）

- [ ] **Step 1: SERVER_DEPLOYMENT.md 加节**

```markdown
### 指定首个管理员

首次部署后，管理员用你的登录邮箱执行：

docker compose exec postgres psql -U storymoss -d storymoss \
  -c "UPDATE users SET role='admin' WHERE email='你的邮箱';"

之后可在网站 /admin 的「管理员」页签提拔其他管理员。
```

- [ ] **Step 2: CHANGELOG v0.34.0 条目增补**

在既有 v0.34.0 条目内追加两节（不新建版本号）：

- 「新增：网站会员系统与管理后台」——migration 004（users.role/disabled_at、invite_codes.grant_pro_days/revoked_at）；Admin API（用户管理/订阅代调/邀请码 CRUD，require_admin 查库 + 审计日志）；web /admin 三页签（邀请码/用户/管理员）+ Dashboard 订阅卡片
- 「修复：订阅与登录安全」——JWT 吊销生效（session+disabled 查库，logout/禁用立即踢下线）；expires_at 全链路生效（/me 过期懒降级、邀请码赠 Pro N 天、桌面缓存透传不再恒 30 天）；作废邀请码注册被拒
- 「测试」节更新数字

- [ ] **Step 3: 全量验证**

- `cd src-server && cargo test`（基线 10 + 新增）+ `cargo +nightly fmt`
- `cd src-tauri && cargo test --lib`（基线 1216 + 新增）+ fmt
- `cd src-server-web && npm test && npm run build`
- `cd src-frontend && npx vitest run && npx tsc --noEmit`（应零改动，确认无回归即可）
- prettier 改动文件

- [ ] **Step 4: Commit（不推送不打 tag）**

```bash
git add CHANGELOG.md SERVER_DEPLOYMENT.md docs/plans/2026-08-09-web-membership-admin-design.md
git commit -m "文档: v0.34.0 增补网站会员系统（Admin 后台+邀请码赠 Pro+JWT 吊销）"
```

- [ ] **Step 5: 手动验证清单（告知用户，server 部署后）**

1. 部署 → 注册登录 → SQL 指定自己为 admin → 访问 /admin 三页签
2. 生成 1 个赠 Pro 30 天邀请码 → 新账号（可隐身窗口另一 provider）用码注册 → Dashboard 显示「专业版 · 有效期至 30 天后」→ 桌面端登录同账号 Pro 生效
3. 禁用该测试账号 → 其 Dashboard 立即 401 跳登录；启用后可登录
4. 作废邀请码 → 新用户用该码注册被拒（桌面端即时提示「邀请码无效或已被使用」）
5. 提拔/降级另一个管理员；尝试降级自己 → 400

---

## Self-Review 记录

- 规格覆盖：设计 ①②③④⑤ ↔ Task 4/1/3/6-8/各测试节 + Task 9 文档；backlog I2（Task 2/3/5）、I3（Task 1）、#2（Task 2 upsert 读回）、#12（Task 5）、#4（Task 4 HTTP 层测试）全部有落点
- 类型一致性：`authenticate(req)`、`upsert_tier(pool, uid, tier, days: Option<i64>)`、`AdminUser{user_id}`、`admin.ts`/`subscription.ts` 导出在 Task 1/2/4/6/7/8 间签名一致；前端 `/api/admin/*` 路径与 server 嵌套 scope（/api/admin）对齐
- 已知取舍：AdminLayout 用 localStorage sf_role 做守卫（UI 层，真正鉴权在 server require_admin）；邀请码格式 SM-+8hex（碰撞由 PK 兜底重试一次）；web 占位页在 Task 6 先建、Task 7/8 填充
