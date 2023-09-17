use std::sync::Arc;

use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    RequestExt,
};
use oxide_auth_axum::{OAuthResource, WebError};
use tracing::{debug, error};

use crate::context::Context;

pub async fn auth_required_middleware<B: Send + 'static>(
    State(state): State<Arc<Context>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let resource_request = match request.extract_parts::<OAuthResource>().await {
        Ok(v) => v,
        Err(e) => {
            error!("Rejecting request due to invalid Authorization header");
            return e.into_response();
        }
    };

    let grant = match state.oauth2.resource(resource_request.into()) {
        Ok(v) => v,
        Err(e) => {
            error!("Rejecting request due to it being unauthorized");
            return e.map_err(|e| e.pack::<WebError>()).into_response();
        }
    };

    debug!(?grant, "Request authorized");

    next.run(request).await
}
