use std::{borrow::Cow, collections::HashMap, sync::Arc};

use axum::{body::Bytes, extract::State, Extension};
use jmap_proto::{
    common::SessionState,
    endpoints::{Argument, Arguments, Invocation, Request, Response},
    errors::MethodError,
};
use oxide_auth::primitives::grant::Grant;

use crate::{
    context::Context,
    extensions::{ConcreteData, ResolvedArguments},
    store::UserProvider,
};

pub async fn handle(
    State(context): State<Arc<Context>>,
    Extension(grant): Extension<Grant>,
    body: Bytes,
) {
    let payload: Request<'_> = serde_json::from_slice(&body).unwrap();

    // TODO: `using`
    // TODO: `method_calls`
    // TODO: `created_ids`

    let username = grant.owner_id;

    let user = context
        .store
        .get_by_username(&username)
        .await
        .unwrap()
        .unwrap();

    let session_state = context
        .store
        .fetch_seq_number_for_user(user.id)
        .await
        .unwrap();

    let mut response = Response {
        method_responses: Vec::with_capacity(payload.method_calls.len()),
        created_ids: None,
        session_state: SessionState(session_state.to_string().into()),
    };

    for invocation_request in payload.method_calls {
        let Some(resolved_arguments) = resolve_arguments(&response, invocation_request.arguments)
        else {
            response.method_responses.push(
                MethodError::InvalidResultReference.into_invocation(invocation_request.request_id),
            );
            continue;
        };

        let Some(_request) =
            ConcreteData::parse(invocation_request.name.as_ref(), resolved_arguments)
        else {
            response
                .method_responses
                .push(MethodError::UnknownMethod.into_invocation(invocation_request.request_id));
            continue;
        };

        // TODO: call handler

        response.method_responses.push(Invocation {
            name: invocation_request.name,
            arguments: Arguments(HashMap::new()),
            request_id: invocation_request.request_id,
        });
    }
}

fn resolve_arguments<'a>(
    response: &'a Response,
    args: Arguments<'a>,
) -> Option<ResolvedArguments<'a>> {
    let mut res = HashMap::with_capacity(args.0.len());

    for (key, value) in args.0 {
        let value = match value {
            Argument::Reference(refer) => {
                let referenced_response = response
                    .method_responses
                    .iter()
                    .find(|inv| inv.request_id == refer.result_of && inv.name == refer.name)?;

                referenced_response.arguments.pointer(&refer.path)?
            }
            Argument::Absolute(value) => Cow::Owned(value),
        };

        res.insert(key, value);
    }

    Some(ResolvedArguments(res))
}
