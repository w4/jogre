use std::sync::Arc;

use axum::extract::State;
use oxide_auth::frontends::simple::endpoint;
use oxide_auth_axum::{OAuthResponse, WebError};

use crate::context::{oauth2::OAuthRequestWrapper, Context};

pub async fn handle(
    State(context): State<Arc<Context>>,
    request: OAuthRequestWrapper,
) -> Result<OAuthResponse, WebError> {
    context
        .oauth2
        .authorize(request)
        .await
        .map_err(endpoint::Error::pack)
}
