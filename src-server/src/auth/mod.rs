//! Authentication Module — Server-side

use serde::{Deserialize, Serialize};

pub mod handlers;
pub mod jwt;
pub mod oauth;

/// OAuth Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Github,
    Wechat,
    Qq,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "google"),
            OAuthProvider::Github => write!(f, "github"),
            OAuthProvider::Wechat => write!(f, "wechat"),
            OAuthProvider::Qq => write!(f, "qq"),
        }
    }
}

impl std::str::FromStr for OAuthProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProvider::Google),
            "github" => Ok(OAuthProvider::Github),
            "wechat" => Ok(OAuthProvider::Wechat),
            "qq" => Ok(OAuthProvider::Qq),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// OAuth 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_account_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

/// 用户信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
