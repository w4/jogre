mod config;
mod context;
mod layers;
mod methods;
mod store;
mod util;

use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use rand::RngCore;
use tracing::info;

use crate::{context::Context, store::UserProvider};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Path to the config file (eg. config.toml)
    #[clap(long, short)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::fmt();
    #[cfg(debug_assertions)]
    let subscriber = subscriber.pretty();
    subscriber.init();

    let config = toml::from_str(&tokio::fs::read_to_string(&args.config).await?)?;

    let context = Arc::new(Context::new(config));

    create_root_if_none_exists(&context).await;

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(methods::router(context).into_make_service())
        .await?;

    Ok(())
}

async fn create_root_if_none_exists(context: &Context) {
    if context.store.has_any_users().await.unwrap() {
        return;
    }

    let mut password = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut password);
    let password = hex::encode(password);

    info!("User root created with password {password}");

    let root = store::User::new("root".into(), &password);
    context.store.create_user(root).await.unwrap();
}