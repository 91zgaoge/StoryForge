//! Admin API — 会员系统管理后台（require_admin 查库校验）

use std::{future::Future, pin::Pin};

use actix_web::{
    dev::Payload, error, get, post, web, FromRequest, HttpRequest, HttpResponse, Responder,
};
use serde_json::json;
use sqlx::PgPool;

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

#[derive(serde::Serialize)]
struct AdminUserItem {
    id: uuid::Uuid,
    email: Option<String>,
    display_name: Option<String>,
    role: String,
    tier: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    disabled_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize)]
struct ListUsersQuery {
    q: Option<String>,
}

#[get("/users")]
async fn list_users(
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListUsersQuery>,
) -> impl Responder {
    let q = query.q.as_deref().filter(|s| !s.is_empty());
    let rows = sqlx::query!(
        r#"SELECT u.id, u.email, u.display_name, u.role, u.disabled_at, u.created_at,
                  s.tier AS "tier?", s.expires_at AS sub_expires_at
           FROM users u LEFT JOIN subscriptions s ON s.user_id = u.id
           WHERE ($1::text IS NULL OR u.email ILIKE '%'||$1||'%' OR u.display_name ILIKE '%'||$1||'%')
           ORDER BY u.created_at DESC LIMIT 200"#,
        q
    )
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|r| AdminUserItem {
                    id: r.id,
                    email: r.email,
                    display_name: r.display_name,
                    role: r.role,
                    tier: r.tier,
                    expires_at: r.sub_expires_at,
                    disabled_at: r.disabled_at,
                    created_at: r.created_at,
                })
                .collect::<Vec<_>>(),
        ),
        Err(e) => {
            log::error!("admin list users failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct SetRoleRequest {
    role: String,
}

#[post("/users/{id}/role")]
async fn set_user_role(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<SetRoleRequest>,
) -> impl Responder {
    let role = body.role.as_str();
    if !["admin", "user"].contains(&role) {
        return HttpResponse::BadRequest().json(json!({"error": "invalid role"}));
    }
    let id = path.into_inner();
    if id == admin.user_id && role != "admin" {
        return HttpResponse::BadRequest().json(json!({"error": "cannot demote self"}));
    }
    match sqlx::query!("UPDATE users SET role = $1 WHERE id = $2", role, id)
        .execute(pool.get_ref())
        .await
    {
        Ok(res) if res.rows_affected() == 0 => {
            HttpResponse::NotFound().json(json!({"error": "user not found"}))
        }
        Ok(_) => {
            log::info!("[admin] {} set role of {} to {}", admin.user_id, id, role);
            HttpResponse::Ok().json(json!({"ok": true}))
        }
        Err(e) => {
            log::error!("admin set role failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[post("/users/{id}/disable")]
async fn disable_user(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("admin disable tx begin failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error": "Internal error"}));
        }
    };
    let res = match sqlx::query!("UPDATE users SET disabled_at = NOW() WHERE id = $1", id)
        .execute(&mut *tx)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            log::error!("admin disable user failed: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error": "Internal error"}));
        }
    };
    if res.rows_affected() == 0 {
        return HttpResponse::NotFound().json(json!({"error": "user not found"}));
    }
    if let Err(e) = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await
    {
        log::error!("admin disable delete sessions failed: {}", e);
        return HttpResponse::InternalServerError().json(json!({"error": "Internal error"}));
    }
    if let Err(e) = tx.commit().await {
        log::error!("admin disable tx commit failed: {}", e);
        return HttpResponse::InternalServerError().json(json!({"error": "Internal error"}));
    }
    log::info!("[admin] {} disabled user {}", admin.user_id, id);
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[post("/users/{id}/enable")]
async fn enable_user(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match sqlx::query!("UPDATE users SET disabled_at = NULL WHERE id = $1", id)
        .execute(pool.get_ref())
        .await
    {
        Ok(res) if res.rows_affected() == 0 => {
            HttpResponse::NotFound().json(json!({"error": "user not found"}))
        }
        Ok(_) => {
            log::info!("[admin] {} enabled user {}", admin.user_id, id);
            HttpResponse::Ok().json(json!({"ok": true}))
        }
        Err(e) => {
            log::error!("admin enable user failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct SetSubscriptionRequest {
    tier: String,
    days: Option<i64>,
}

#[post("/users/{id}/subscription")]
async fn set_user_subscription(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<SetSubscriptionRequest>,
) -> impl Responder {
    let tier = body.tier.as_str();
    if !["pro", "free", "enterprise"].contains(&tier) {
        return HttpResponse::BadRequest().json(json!({"error": "invalid tier"}));
    }
    if let Some(days) = body.days {
        if days <= 0 {
            return HttpResponse::BadRequest().json(json!({"error": "days must be positive"}));
        }
    }
    let id = path.into_inner();
    match crate::api::subscription::upsert_tier(pool.get_ref(), id, tier, body.days).await {
        Ok(row) => {
            log::info!(
                "[admin] {} set subscription of {} to {} (days: {:?})",
                admin.user_id,
                id,
                tier,
                body.days
            );
            HttpResponse::Ok().json(json!({
                "user_id": id,
                "tier": row.tier,
                "status": row.status,
                "expires_at": row.expires_at.map(|t| t.to_rfc3339()),
            }))
        }
        Err(e) => {
            log::error!("admin set subscription failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Serialize)]
struct InviteCodeItem {
    code: String,
    max_uses: i32,
    used_count: i32,
    grant_pro_days: Option<i32>,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[get("/invite-codes")]
async fn list_invite_codes(_admin: AdminUser, pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query!(
        "SELECT code, max_uses, used_count, grant_pro_days, note, created_at, revoked_at
         FROM invite_codes ORDER BY created_at DESC LIMIT 500"
    )
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|r| InviteCodeItem {
                    code: r.code,
                    max_uses: r.max_uses,
                    used_count: r.used_count,
                    grant_pro_days: r.grant_pro_days,
                    note: r.note,
                    created_at: r.created_at,
                    revoked_at: r.revoked_at,
                })
                .collect::<Vec<_>>(),
        ),
        Err(e) => {
            log::error!("admin list invite codes failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[derive(serde::Deserialize)]
struct CreateInviteCodesRequest {
    count: i64,
    max_uses: i32,
    grant_pro_days: Option<i32>,
    note: Option<String>,
}

/// 生成 `SM-` + 8 位大写 hex 邀请码
fn gen_invite_code() -> String {
    format!(
        "SM-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase()
    )
}

#[post("/invite-codes")]
async fn create_invite_codes(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<CreateInviteCodesRequest>,
) -> impl Responder {
    if !(1..=100).contains(&body.count) {
        return HttpResponse::BadRequest().json(json!({"error": "count must be 1..=100"}));
    }
    if body.max_uses < 1 {
        return HttpResponse::BadRequest().json(json!({"error": "max_uses must be >= 1"}));
    }
    let mut codes = Vec::with_capacity(body.count as usize);
    for _ in 0..body.count {
        // 撞 PK（code 唯一）重试一次
        let mut inserted = false;
        for _ in 0..2 {
            let code = gen_invite_code();
            let res = sqlx::query!(
                "INSERT INTO invite_codes (code, max_uses, grant_pro_days, note, created_by)
                 VALUES ($1, $2, $3, $4, $5)",
                code,
                body.max_uses,
                body.grant_pro_days,
                body.note,
                admin.user_id
            )
            .execute(pool.get_ref())
            .await;
            match res {
                Ok(_) => {
                    codes.push(code);
                    inserted = true;
                    break;
                }
                Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => continue,
                Err(e) => {
                    log::error!("admin create invite code failed: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Internal error"}));
                }
            }
        }
        if !inserted {
            log::error!("admin create invite code failed: pk collision twice");
            return HttpResponse::InternalServerError().json(json!({"error": "Internal error"}));
        }
    }
    log::info!(
        "[admin] {} created {} invite codes (max_uses: {}, grant_pro_days: {:?})",
        admin.user_id,
        codes.len(),
        body.max_uses,
        body.grant_pro_days
    );
    HttpResponse::Ok().json(json!({"codes": codes}))
}

#[post("/invite-codes/{code}/revoke")]
async fn revoke_invite_code(
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> impl Responder {
    let code = path.into_inner();
    match sqlx::query!(
        "UPDATE invite_codes SET revoked_at = NOW() WHERE code = $1 AND revoked_at IS NULL",
        code
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound()
            .json(json!({"error": "invite code not found or already revoked"})),
        Ok(_) => {
            log::info!("[admin] {} revoked invite code {}", admin.user_id, code);
            HttpResponse::Ok().json(json!({"ok": true}))
        }
        Err(e) => {
            log::error!("admin revoke invite code failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Internal error"}))
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};

    use super::*;

    async fn seed_user(pool: &PgPool, email: &str, role: &str) -> (uuid::Uuid, String) {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email, role) VALUES ($1, $2, $3)",
            uid,
            email,
            role
        )
        .execute(pool)
        .await
        .unwrap();
        let token = crate::auth::jwt::create_token(&uid.to_string()).unwrap();
        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
            uid, token
        )
        .execute(pool)
        .await
        .unwrap();
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
        let req = test::TestRequest::get()
            .uri("/api/admin/users")
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 401);
        // 普通用户 → 403
        let (_, tok) = seed_user(&pool, "u@t.com", "user").await;
        let req = test::TestRequest::get()
            .uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", tok)))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 403);
        // admin → 200
        let (_, tok) = seed_user(&pool, "a@t.com", "admin").await;
        let req = test::TestRequest::get()
            .uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", tok)))
            .to_request();
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
        let req = test::TestRequest::get()
            .uri("/api/admin/users")
            .insert_header(("Authorization", format!("Bearer {}", victim_tok)))
            .to_request();
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

        let req = test::TestRequest::post()
            .uri("/api/admin/invite-codes")
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
