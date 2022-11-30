#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::option_if_let_else)]

use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use axum::Server;
use tokio::sync::Mutex;
use tokio_shutdown::Shutdown;
use tracing::{info, Level, Subscriber};
use tracing_quiver::Handle;
use tracing_subscriber::{filter::Targets, prelude::*, registry::LookupSpan, Layer};

use crate::{settings::Quiver, twitch::Client};

mod handlers;
mod lang;
mod middleware;
mod routes;
mod settings;
mod templates;
mod twitch;

const ADDRESS: Ipv4Addr = if cfg!(debug_assertions) {
    Ipv4Addr::LOCALHOST
} else {
    Ipv4Addr::UNSPECIFIED
};

#[tokio::main]
async fn main() -> Result<()> {
    let settings = settings::load()?;

    let (quiver, handle) = match settings.tracing.quiver.map(init_tracing) {
        Some(tracing) => {
            let (quiver, handle) = tracing.await?;
            (Some(quiver), Some(handle))
        }
        None => (None, None),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(quiver)
        .with(
            Targets::new()
                .with_target(env!("CARGO_CRATE_NAME"), Level::TRACE)
                .with_target("tower_http", Level::TRACE)
                .with_default(Level::INFO),
        )
        .init();

    let shutdown = Shutdown::new()?;
    let client = Client::new(settings.twitch).await?;
    let client = Arc::new(Mutex::new(client));

    let addr = SocketAddr::from((ADDRESS, 8080));
    let app = routes::build(&settings.server).with_state(client);

    let server = Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown.handle());

    info!("listening on http://{addr}");

    server.await?;

    if let Some(handle) = handle {
        handle.shutdown(Duration::from_secs(5)).await;
    }

    Ok(())
}

async fn init_tracing<S>(settings: Quiver) -> Result<(impl Layer<S>, Handle)>
where
    for<'span> S: Subscriber + LookupSpan<'span>,
{
    tracing_quiver::builder()
        .with_server_addr(settings.address)
        .with_server_cert(settings.certificate)
        .with_resource(env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_VERSION"))
        .build()
        .await
        .map_err(Into::into)
}
