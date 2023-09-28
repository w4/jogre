mod api;
mod oauth;
mod session;

use std::sync::Arc;

use axum::{
    routing::{any, get},
    Router,
};
use tower::layer::layer_fn;
use tower_cookies::CookieManagerLayer;

use crate::{
    context::Context,
    layers::{auth_required::auth_required_middleware, logger::LoggingMiddleware},
};

pub fn router(context: Arc<Context>) -> Router {
    Router::new()
        .route("/.well-known/jmap", get(session::get))
        .route("/api/*", any(api::handle))
        // only apply auth requirement on endpoints above
        .layer(axum::middleware::from_fn_with_state(
            context.clone(),
            auth_required_middleware,
        ))
        .nest("/oauth", oauth::router())
        .layer(layer_fn(LoggingMiddleware))
        .layer(CookieManagerLayer::new())
        .with_state(context)
}
