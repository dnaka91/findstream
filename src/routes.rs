use std::{sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use crate::{handlers, middleware, settings, twitch::AsyncClient};

pub fn build(client: AsyncClient, settings: &settings::Server) -> Router<AsyncClient> {
    Router::with_state(Arc::clone(&client))
        .nest("/api", api(client))
        .route("/favicon.svg", get(handlers::favicon))
        .route("/api-info", get(handlers::api_info))
        .route("/", get(handlers::index))
        .route("/search", get(handlers::search))
        .layer(
            ServiceBuilder::new()
                .map_response(middleware::add_vary_header)
                .layer(HandleErrorLayer::new(handlers::error))
                .load_shed()
                .concurrency_limit(settings.concurrency_limit.unwrap_or(100))
                .timeout(settings.timeout.unwrap_or(Duration::from_secs(15)))
                .trace_for_http()
                .compression(),
        )
}

fn api(client: AsyncClient) -> Router<AsyncClient> {
    Router::with_state(client).route("/search", post(handlers::api::search))
}
