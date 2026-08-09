//! JWT 签发与验证

use std::{future::Future, pin::Pin};

use actix_web::{dev::Payload, error, web, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

const TOKEN_EXPIRY_DAYS: i64 = 7;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    pub sub: String, // user_id
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

/// 生成JWT token
pub fn create_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(TOKEN_EXPIRY_DAYS);
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = AuthClaims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
    )
}

/// 验证JWT token
pub fn validate_token(token: &str) -> Result<AuthClaims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let decoded = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
        &validation,
    )?;
    Ok(decoded.claims)
}

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

/// Actix-web 提取器：从请求头中获取JWT
impl FromRequest for AuthClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { authenticate(&req).await })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{dev::Payload, test::TestRequest, web, FromRequest};

    use super::*;

    async fn seed_user_session(pool: &sqlx::PgPool, token: &str) -> uuid::Uuid {
        let uid = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            uid,
            "jwt@t.com"
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
            uid, token
        )
        .execute(pool)
        .await
        .unwrap();
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
        sqlx::query!(
            "UPDATE sessions SET token = $1 WHERE user_id = $2",
            token,
            uid
        )
        .execute(&pool)
        .await
        .unwrap();
        let req = req_with(&pool, &token);
        let claims = AuthClaims::from_request(&req, &mut Payload::None)
            .await
            .unwrap();
        assert_eq!(claims.sub, uid.to_string());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn revoked_session_is_rejected(pool: sqlx::PgPool) {
        let uid = seed_user_session(&pool, "tok-x").await;
        let token = create_token(&uid.to_string()).unwrap();
        // 不把 token 写入 sessions（= 已注销/从未登记）
        let req = req_with(&pool, &token);
        assert!(AuthClaims::from_request(&req, &mut Payload::None)
            .await
            .is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn disabled_user_is_rejected(pool: sqlx::PgPool) {
        let uid = seed_user_session(&pool, "tok-y").await;
        let token = create_token(&uid.to_string()).unwrap();
        sqlx::query!(
            "UPDATE sessions SET token = $1 WHERE user_id = $2",
            token,
            uid
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!("UPDATE users SET disabled_at = NOW() WHERE id = $1", uid)
            .execute(&pool)
            .await
            .unwrap();
        let req = req_with(&pool, &token);
        assert!(AuthClaims::from_request(&req, &mut Payload::None)
            .await
            .is_err());
    }
}
