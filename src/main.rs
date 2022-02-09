#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::option_if_let_else)]

use std::{
    env,
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use axum::{routing::IntoMakeService, Router, Server};
use tokio::sync::Mutex;
use tokio_shutdown::Shutdown;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, prelude::*};

use crate::twitch::Client;

mod handlers;
mod lang;
mod responses;
mod routes;
mod settings;
mod templates;
mod twitch;

const ADDRESS_V4: Ipv4Addr = if cfg!(debug_assertions) {
    Ipv4Addr::LOCALHOST
} else {
    Ipv4Addr::UNSPECIFIED
};

const ADDRESS_V6: Ipv6Addr = if cfg!(debug_assertions) {
    Ipv6Addr::LOCALHOST
} else {
    Ipv6Addr::UNSPECIFIED
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            Targets::new()
                .with_target(env!("CARGO_CRATE_NAME"), Level::TRACE)
                .with_target("tower_http", Level::TRACE)
                .with_default(Level::INFO),
        )
        .init();

    let settings = settings::load()?;
    let client = Client::new(settings.client_id, settings.client_secret).await?;
    let client = Arc::new(Mutex::new(client));

    let app = routes::build(client).into_make_service();

    let shutdown = Shutdown::new()?;

    let server_v4 = build_server(ADDRESS_V4.into(), &shutdown, app.clone())?;
    let server_v6 = build_server(ADDRESS_V6.into(), &shutdown, app)?;

    let (res_v4, res_v6) = tokio::join!(tokio::spawn(server_v4), tokio::spawn(server_v6));

    res_v4??;
    res_v6??;

    Ok(())
}

fn build_server(
    addr: IpAddr,
    shutdown: &Shutdown,
    service: IntoMakeService<Router>,
) -> Result<impl Future<Output = Result<()>>> {
    let addr = SocketAddr::from((addr, 8080));

    let server = Server::try_bind(&addr)?
        .serve(service)
        .with_graceful_shutdown(shutdown.handle());

    Ok(async move {
        info!("Listening on {}", addr);
        server.await.map_err(Into::into)
    })
}
