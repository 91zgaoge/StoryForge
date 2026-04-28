//! API Routes

use actix_web::web;

mod health;
mod user;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(health::init_routes)
            .configure(user::init_routes)
            .configure(super::auth::handlers::init_routes),
    );
}
