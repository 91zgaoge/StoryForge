//! User API

use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;

use crate::auth::jwt::AuthClaims;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me);
}

#[get("/users/me")]
async fn get_me(
    claims: AuthClaims,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user = sqlx::query!(
        "SELECT id, email, display_name, avatar_url, created_at FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(row)) => {
            HttpResponse::Ok().json(json!({
                "id": row.id,
                "email": row.email,
                "display_name": row.display_name,
                "avatar_url": row.avatar_url,
                "created_at": row.created_at,
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "User not found"
        })),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Internal server error"
            }))
        }
    }
}
