use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use crate::{handlers, twitch::AsyncClient};

pub fn build(client: AsyncClient) -> Router {
    Router::new()
        .nest("/api", api())
        .route("/favicon-16x16.png", get(handlers::favicon_16))
        .route("/favicon-32x32.png", get(handlers::favicon_32))
        .route("/api-info", get(handlers::api_info))
        .route("/", get(handlers::index))
        .route("/search", get(handlers::search))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handlers::error))
                .load_shed()
                .concurrency_limit(100)
                .timeout(Duration::from_secs(15))
                .trace_for_http()
                .compression()
                .layer(Extension(client))
                .into_inner(),
        )
}

fn api() -> Router {
    Router::new().route("/search", post(handlers::api::search))
}
