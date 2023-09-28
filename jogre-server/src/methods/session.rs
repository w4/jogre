use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use axum::{extract::State, Extension, Json};
use jmap_proto::{
    common::{Id, SessionState},
    endpoints::session::{Account, AccountCapabilities, Session},
};
use oxide_auth::primitives::grant::Grant;

use crate::{
    context::Context,
    store::{AccountProvider, UserProvider},
};

static API_URL: OnceLock<Box<str>> = OnceLock::new();
static DOWNLOAD_URL: OnceLock<Box<str>> = OnceLock::new();
static UPLOAD_URL: OnceLock<Box<str>> = OnceLock::new();
static EVENT_SOURCE_URL: OnceLock<Box<str>> = OnceLock::new();

pub async fn get(
    State(context): State<Arc<Context>>,
    Extension(grant): Extension<Grant>,
) -> Json<Session<'static>> {
    let username = grant.owner_id;

    let user = context
        .store
        .get_by_username(&username)
        .await
        .unwrap()
        .unwrap();

    let (accounts, user_seq_number) = tokio::join!(
        async {
            context
                .store
                .get_accounts_for_user(user.id)
                .await
                .unwrap()
                .into_iter()
                .map(|acc| {
                    (
                        Id(acc.id.to_string().into()),
                        Account {
                            name: acc.name.into(),
                            is_personal: acc.is_personal,
                            is_read_only: acc.is_read_only,
                            account_capabilities: AccountCapabilities {},
                        },
                    )
                })
                .collect()
        },
        async {
            context
                .store
                .fetch_seq_number_for_user(user.id)
                .await
                .unwrap()
        }
    );

    Json(Session {
        capabilities: context
            .extension_registry
            .build_session_capabilities(user.id),
        accounts,
        primary_accounts: HashMap::default(),
        username: username.into(),
        api_url: API_URL
            .get_or_init(|| {
                context
                    .base_url
                    .join("api/")
                    .unwrap()
                    .to_string()
                    .into_boxed_str()
            })
            .as_ref()
            .into(),
        download_url: DOWNLOAD_URL
            .get_or_init(|| {
                let base = context.base_url.join("download/").unwrap();
                format!("{base}{{accountId}}/{{blobId}}/{{name}}?accept={{type}}").into_boxed_str()
            })
            .as_ref()
            .into(),
        upload_url: UPLOAD_URL
            .get_or_init(|| {
                let base = context.base_url.join("upload/").unwrap();
                format!("{base}{{accountId}}/").into_boxed_str()
            })
            .as_ref()
            .into(),
        event_source_url: EVENT_SOURCE_URL
            .get_or_init(|| {
                context
                    .base_url
                    .join("eventsource/?types={types}&closeafter={closeafter}&ping={ping}")
                    .unwrap()
                    .to_string()
                    .into_boxed_str()
            })
            .as_ref()
            .into(),
        state: SessionState(user_seq_number.to_string().into()),
    })
}
