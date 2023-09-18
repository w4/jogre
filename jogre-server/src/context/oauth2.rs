use std::{
    borrow::Cow,
    str::FromStr,
    sync::{Arc, Mutex},
};

use askama::Template;
use axum::{
    async_trait,
    body::HttpBody,
    extract::FromRequest,
    http::{Method, Request},
    BoxError, RequestExt,
};
use oxide_auth::{
    endpoint::{OAuthError, OwnerConsent, QueryParameter, Scope, Scopes, Solicitation, WebRequest},
    frontends::simple::{
        endpoint,
        endpoint::{Error, ResponseCreator, Vacant},
    },
    primitives::{
        grant::Grant,
        issuer::{IssuedToken, RefreshedToken},
        prelude::{AuthMap, Client, ClientMap, RandomGenerator, TokenMap},
        registrar::RegisteredUrl,
    },
};
use oxide_auth_async::endpoint::{
    access_token::AccessTokenFlow, authorization::AuthorizationFlow, refresh::RefreshFlow,
    resource::ResourceFlow, OwnerSolicitor,
};
use oxide_auth_axum::{OAuthRequest, OAuthResponse, WebError};
use tower_cookies::Cookies;
use tracing::info;
use url::Url;

use crate::{
    context::DerivedKeys,
    store::{Store, UserProvider},
    util::CsrfToken,
};

pub struct OAuth2 {
    pub registrar: ClientMap,
    pub authorizer: Authorizer,
    pub issuer: Issuer,
    pub derived_keys: Arc<DerivedKeys>,
    pub store: Arc<Store>,
}

impl OAuth2 {
    pub fn new(store: Arc<Store>, derived_keys: Arc<DerivedKeys>) -> Self {
        let mut registrar = ClientMap::new();

        registrar.register_client(Client::public(
            "abcdef",
            RegisteredUrl::from("https://google.com/".parse::<Url>().unwrap()),
            "test".parse::<Scope>().unwrap(),
        ));

        let authorizer = Authorizer::default();
        let issuer = Issuer::default();

        Self {
            registrar,
            authorizer,
            issuer,
            derived_keys,
            store,
        }
    }

    pub async fn resource(
        &self,
        request: OAuthRequest,
    ) -> Result<Grant, Result<OAuthResponse, endpoint::Error<OAuthRequest>>> {
        match ResourceFlow::prepare(self.endpoint()) {
            Ok(mut flow) => flow.execute(request).await,
            Err(e) => Err(Err(e)),
        }
    }

    pub async fn authorize(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        AuthorizationFlow::prepare(self.endpoint())?
            .execute(request)
            .await
    }

    pub async fn token(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        AccessTokenFlow::prepare(self.endpoint())?
            .execute(request)
            .await
    }

    pub async fn refresh(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        RefreshFlow::prepare(self.endpoint())?
            .execute(request)
            .await
    }

    fn endpoint(&self) -> Endpoint<'_> {
        Endpoint {
            registrar: &self.registrar,
            authorizer: self.authorizer.clone(),
            issuer: self.issuer.clone(),
            solicitor: Solicitor {
                derived_keys: &self.derived_keys,
                store: &self.store,
            },
            scopes: vec![Scope::from_str("test").unwrap()],
            response: Vacant,
        }
    }
}

pub struct Endpoint<'a> {
    registrar: &'a ClientMap,
    authorizer: Authorizer,
    issuer: Issuer,
    solicitor: Solicitor<'a>,
    scopes: Vec<Scope>,
    response: Vacant,
}

impl<T: WebRequest + Send> oxide_auth_async::endpoint::Endpoint<T> for Endpoint<'_>
where
    <T as WebRequest>::Response: Default,
    for<'a> Solicitor<'a>: OwnerSolicitor<T>,
{
    type Error = Error<T>;

    fn registrar(&self) -> Option<&(dyn oxide_auth_async::primitives::Registrar + Sync)> {
        Some(&self.registrar)
    }

    fn authorizer_mut(
        &mut self,
    ) -> Option<&mut (dyn oxide_auth_async::primitives::Authorizer + Send)> {
        Some(&mut self.authorizer)
    }

    fn issuer_mut(&mut self) -> Option<&mut (dyn oxide_auth_async::primitives::Issuer + Send)> {
        Some(&mut self.issuer)
    }

    fn owner_solicitor(&mut self) -> Option<&mut (dyn OwnerSolicitor<T> + Send)> {
        Some(&mut self.solicitor)
    }

    fn scopes(&mut self) -> Option<&mut dyn Scopes<T>> {
        Some(&mut self.scopes)
    }

    fn response(
        &mut self,
        request: &mut T,
        kind: oxide_auth::endpoint::Template,
    ) -> Result<T::Response, Self::Error> {
        Ok(self.response.create(request, kind))
    }

    fn error(&mut self, err: OAuthError) -> Self::Error {
        Error::OAuth(err)
    }

    fn web_error(&mut self, err: T::Error) -> Self::Error {
        Error::Web(err)
    }
}

#[derive(Clone)]
pub struct Issuer {
    issuer: Arc<Mutex<TokenMap<RandomGenerator>>>,
}

impl Default for Issuer {
    fn default() -> Self {
        Self {
            issuer: Arc::new(Mutex::new(TokenMap::new(RandomGenerator::new(16)))),
        }
    }
}

#[async_trait]
impl oxide_auth_async::primitives::Issuer for Issuer {
    async fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        oxide_auth::primitives::issuer::Issuer::issue(&mut self.issuer.lock().unwrap(), grant)
    }

    async fn refresh(&mut self, token: &str, grant: Grant) -> Result<RefreshedToken, ()> {
        oxide_auth::primitives::issuer::Issuer::refresh(
            &mut self.issuer.lock().unwrap(),
            token,
            grant,
        )
    }

    async fn recover_token(&mut self, token: &str) -> Result<Option<Grant>, ()> {
        oxide_auth::primitives::issuer::Issuer::recover_token(&self.issuer.lock().unwrap(), token)
    }

    async fn recover_refresh(&mut self, token: &str) -> Result<Option<Grant>, ()> {
        oxide_auth::primitives::issuer::Issuer::recover_refresh(&self.issuer.lock().unwrap(), token)
    }
}

#[derive(Clone)]
pub struct Authorizer {
    auth: Arc<Mutex<AuthMap<RandomGenerator>>>,
}

impl Default for Authorizer {
    fn default() -> Self {
        Self {
            auth: Arc::new(Mutex::new(AuthMap::new(RandomGenerator::new(16)))),
        }
    }
}

#[async_trait]
impl oxide_auth_async::primitives::Authorizer for Authorizer {
    async fn authorize(&mut self, grant: Grant) -> Result<String, ()> {
        oxide_auth::primitives::authorizer::Authorizer::authorize(
            &mut self.auth.lock().unwrap(),
            grant,
        )
    }

    async fn extract(&mut self, token: &str) -> Result<Option<Grant>, ()> {
        oxide_auth::primitives::authorizer::Authorizer::extract(
            &mut self.auth.lock().unwrap(),
            token,
        )
    }
}

pub struct Solicitor<'a> {
    derived_keys: &'a DerivedKeys,
    store: &'a Store,
}

#[async_trait]
impl OwnerSolicitor<OAuthRequest> for Solicitor<'_> {
    async fn check_consent(
        &mut self,
        _: &mut OAuthRequest,
        _: Solicitation<'_>,
    ) -> OwnerConsent<OAuthResponse> {
        unreachable!("OAuthRequest should only be used for resource requests")
    }
}

#[async_trait]
impl OwnerSolicitor<OAuthRequestWrapper> for Solicitor<'_> {
    async fn check_consent(
        &mut self,
        req: &mut OAuthRequestWrapper,
        solicitation: Solicitation<'_>,
    ) -> OwnerConsent<OAuthResponse> {
        let auth_state = if req.method == Method::GET {
            AuthState::Unauthenticated(None)
        } else if let Some(((username, password), csrf_token)) = req.inner.body().and_then(|body| {
            body.unique_value("username")
                .zip(body.unique_value("password"))
                .zip(body.unique_value("csrf_token"))
        }) {
            attempt_authentication(
                self.derived_keys,
                self.store,
                &req.cookie_jar,
                &username,
                password.into_owned(),
                &csrf_token,
            )
            .await
        } else {
            AuthState::Unauthenticated(Some(UnauthenticatedState::MissingUserPass))
        };

        match auth_state {
            AuthState::Unauthenticated(reason) => {
                info!("Soliciting auth from user due to {reason:?}");

                let csrf_token = CsrfToken::new(self.derived_keys);
                csrf_token.write_cookie(&req.cookie_jar);

                let response = OAuthResponse::default()
                    .content_type("text/html")
                    .unwrap()
                    .body(
                        &LoginForm {
                            reason,
                            csrf_token,
                            solicitation,
                        }
                        .render()
                        .unwrap(),
                    );

                OwnerConsent::InProgress(response)
            }
            AuthState::Authenticated(username) => OwnerConsent::Authorized(username),
        }
    }
}

async fn attempt_authentication(
    derived_keys: &DerivedKeys,
    store: &Store,
    cookies: &Cookies,
    username: &str,
    password: String,
    csrf_token: &str,
) -> AuthState {
    if !CsrfToken::verify(derived_keys, cookies, csrf_token) {
        return AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidCsrfToken));
    }

    // TODO: actually await here
    let Some(user) = store.get_by_username(username).await.unwrap() else {
        return AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidUserPass));
    };

    tokio::task::spawn_blocking(move || {
        if user.verify_password(&password) {
            AuthState::Authenticated(user.username)
        } else {
            AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidUserPass))
        }
    })
    .await
    .unwrap()
}

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginForm<'a> {
    reason: Option<UnauthenticatedState>,
    csrf_token: CsrfToken,
    solicitation: Solicitation<'a>,
}

pub enum AuthState {
    Authenticated(String),
    Unauthenticated(Option<UnauthenticatedState>),
}

#[derive(Debug)]
pub enum UnauthenticatedState {
    InvalidUserPass,
    MissingUserPass,
    InvalidCsrfToken,
}

pub struct OAuthRequestWrapper {
    inner: OAuthRequest,
    method: Method,
    cookie_jar: Cookies,
}

impl WebRequest for OAuthRequestWrapper {
    type Error = WebError;
    type Response = OAuthResponse;

    fn query(&mut self) -> Result<Cow<dyn QueryParameter + 'static>, Self::Error> {
        WebRequest::query(&mut self.inner)
    }

    fn urlbody(&mut self) -> Result<Cow<dyn QueryParameter + 'static>, Self::Error> {
        WebRequest::urlbody(&mut self.inner)
    }

    fn authheader(&mut self) -> Result<Option<Cow<str>>, Self::Error> {
        WebRequest::authheader(&mut self.inner)
    }
}

#[async_trait]
impl<S, B> FromRequest<S, B> for OAuthRequestWrapper
where
    B: HttpBody + Send + Sync + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request(mut req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self {
            method: req.method().clone(),
            cookie_jar: req.extract_parts_with_state(state).await.unwrap(),
            inner: OAuthRequest::from_request(req, state).await?,
        })
    }
}
