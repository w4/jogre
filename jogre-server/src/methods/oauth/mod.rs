mod authorize;
mod refresh;
mod token;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::context::Context;

pub fn router() -> Router<Arc<Context>> {
    Router::new()
        .route("/authorize", get(authorize::handle).post(authorize::handle))
        .route("/token", post(token::handle))
        .route("/refresh", post(refresh::handle))
}
