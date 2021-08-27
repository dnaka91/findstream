#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::option_if_let_else)]

use std::{env, net::SocketAddr, sync::Arc};

use anyhow::Result;
use hyper::Server;
use tokio::{signal, sync::Mutex};
use tracing::info;

use crate::twitch::TwitchClient;

mod routes;
mod settings;
mod templates;
mod twitch;

#[cfg(debug_assertions)]
const ADDRESS: [u8; 4] = [127, 0, 0, 1];
#[cfg(not(debug_assertions))]
const ADDRESS: [u8; 4] = [0, 0, 0, 0];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "findstream=trace,warn");
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let settings = settings::load()?;
    let client = TwitchClient::new(settings.client_id, settings.client_secret).await?;
    let client = Arc::new(Mutex::new(client));

    let addr = SocketAddr::from((ADDRESS, 8080));

    let server = Server::try_bind(&addr)?
        .serve(routes::build(client).into_make_service())
        .with_graceful_shutdown(shutdown());

    info!("Listening on {}", addr);

    server.await?;

    Ok(())
}

async fn shutdown() {
    signal::ctrl_c().await.ok();
    info!("Shutting down");
}
