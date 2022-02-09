use axum::{
    routing::{get, post},
    AddExtensionLayer, Router,
};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

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
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(AddExtensionLayer::new(client))
                .into_inner(),
        )
}

fn api() -> Router {
    Router::new().route("/search", post(handlers::api::search))
}
