#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::option_if_let_else)]

use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use axum::Server;
use settings::Otlp;
use tokio::sync::Mutex;
use tokio_shutdown::Shutdown;
use tracing::{error, info, Level, Subscriber};
use tracing_subscriber::{filter::Targets, prelude::*, registry::LookupSpan, Layer};

use crate::twitch::Client;

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

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(settings.tracing.otlp.map(init_tracing).transpose()?)
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

    let server = Server::try_bind(&addr)?
        .serve(routes::build(client, &settings.server).into_make_service())
        .with_graceful_shutdown(shutdown.handle());

    info!("listening on http://{addr}");

    server.await?;

    Ok(())
}

fn init_tracing<S>(settings: Otlp) -> Result<impl Layer<S>>
where
    for<'span> S: Subscriber + LookupSpan<'span>,
{
    use opentelemetry::{
        global, runtime,
        sdk::{trace, Resource},
    };
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_semantic_conventions::resource;

    global::set_error_handler(|error| {
        error!(target:"opentelemetry", %error);
    })?;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(settings.endpoint),
        )
        .with_trace_config(trace::config().with_resource(Resource::new([
            resource::SERVICE_NAME.string(env!("CARGO_CRATE_NAME")),
            resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
        ])))
        .install_batch(runtime::Tokio)?;

    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}
