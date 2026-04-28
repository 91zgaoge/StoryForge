//! JWT 签发与验证

use actix_web::{dev::Payload, error, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

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

/// Actix-web 提取器：从请求头中获取JWT
impl FromRequest for AuthClaims {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        match token {
            Some(t) => {
                let result = validate_token(t);
                match result {
                    Ok(claims) => Box::pin(async move { Ok(claims) }),
                    Err(_) => Box::pin(async move {
                        Err(error::ErrorUnauthorized("Invalid token"))
                    }),
                }
            }
            None => Box::pin(async move {
                Err(error::ErrorUnauthorized("Missing Authorization header"))
            }),
        }
    }
}
