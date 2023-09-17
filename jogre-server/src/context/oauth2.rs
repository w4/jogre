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
use futures::FutureExt;
use oxide_auth::{
    endpoint::{
        Authorizer, Issuer, OwnerConsent, OwnerSolicitor, QueryParameter, Registrar, Scope,
        Solicitation, WebRequest,
    },
    frontends::simple::{
        endpoint,
        endpoint::{Generic, Vacant},
    },
    primitives::{
        grant::Grant,
        prelude::{AuthMap, Client, ClientMap, RandomGenerator, TokenMap},
        registrar::RegisteredUrl,
    },
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
    pub authorizer: Mutex<AuthMap<RandomGenerator>>,
    pub issuer: Mutex<TokenMap<RandomGenerator>>,
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

        let authorizer = Mutex::new(AuthMap::new(RandomGenerator::new(16)));
        let issuer = Mutex::new(TokenMap::new(RandomGenerator::new(16)));

        Self {
            registrar,
            authorizer,
            issuer,
            derived_keys,
            store,
        }
    }

    pub fn resource(
        &self,
        request: OAuthRequest,
    ) -> Result<Grant, Result<OAuthResponse, endpoint::Error<OAuthRequest>>> {
        self.endpoint().resource_flow().execute(request)
    }

    pub fn authorize(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        self.endpoint().authorization_flow().execute(request)
    }

    pub fn token(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        self.endpoint().access_token_flow().execute(request)
    }

    pub fn refresh(
        &self,
        request: OAuthRequestWrapper,
    ) -> Result<OAuthResponse, endpoint::Error<OAuthRequestWrapper>> {
        self.endpoint().refresh_flow().execute(request)
    }

    fn endpoint(
        &self,
    ) -> Generic<
        impl Registrar + '_,
        impl Authorizer + '_,
        impl Issuer + '_,
        Solicitor<'_>,
        Vec<Scope>,
    > {
        Generic {
            registrar: &self.registrar,
            authorizer: self.authorizer.lock().unwrap(),
            issuer: self.issuer.lock().unwrap(),
            solicitor: Solicitor {
                derived_keys: &self.derived_keys,
                store: &self.store,
            },
            scopes: vec![Scope::from_str("test").unwrap()],
            response: Vacant,
        }
    }
}

pub struct Solicitor<'a> {
    derived_keys: &'a DerivedKeys,
    store: &'a Store,
}

impl OwnerSolicitor<OAuthRequest> for Solicitor<'_> {
    fn check_consent(
        &mut self,
        _: &mut OAuthRequest,
        _: Solicitation,
    ) -> OwnerConsent<OAuthResponse> {
        unreachable!("OAuthRequest should only be used for resource requests")
    }
}

impl OwnerSolicitor<OAuthRequestWrapper> for Solicitor<'_> {
    fn check_consent(
        &mut self,
        req: &mut OAuthRequestWrapper,
        solicitation: Solicitation,
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
                &password,
                &csrf_token,
            )
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

fn attempt_authentication(
    derived_keys: &DerivedKeys,
    store: &Store,
    cookies: &Cookies,
    username: &str,
    password: &str,
    csrf_token: &str,
) -> AuthState {
    if !CsrfToken::verify(derived_keys, cookies, csrf_token) {
        return AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidCsrfToken));
    }

    // TODO: actually await here
    let Some(user) = store
        .get_by_username(username)
        .now_or_never()
        .unwrap()
        .unwrap()
    else {
        return AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidUserPass));
    };

    if user.verify_password(password) {
        AuthState::Authenticated(user.username)
    } else {
        AuthState::Unauthenticated(Some(UnauthenticatedState::InvalidUserPass))
    }
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
