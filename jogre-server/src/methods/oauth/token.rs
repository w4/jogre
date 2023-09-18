use std::sync::Arc;

use axum::extract::State;
use oxide_auth::frontends::simple::endpoint;
use oxide_auth_axum::{OAuthResponse, WebError};

use crate::context::{oauth2::OAuthRequestWrapper, Context};

#[allow(clippy::unused_async)]
pub async fn handle(
    State(context): State<Arc<Context>>,
    request: OAuthRequestWrapper,
) -> Result<OAuthResponse, WebError> {
    context.oauth2.token(request).map_err(endpoint::Error::pack)
}
