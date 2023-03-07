#![allow(clippy::unused_async)]

pub mod api;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    BoxError,
};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use tracing::{error, instrument};

use crate::{
    templates,
    twitch::{AsyncClient, Category, Stream},
};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    category: Category,
    query: String,
    #[serde(default)]
    language: String,
}

#[instrument]
pub async fn index() -> impl IntoResponse {
    templates::Index
}

#[instrument]
pub async fn api_info() -> impl IntoResponse {
    templates::ApiInfo
}

#[instrument]
pub async fn favicon() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml")],
        include_str!("../../assets/favicon.svg"),
    )
}

#[instrument(skip(client))]
pub async fn search(
    Query(params): Query<SearchParams>,
    State(client): State<AsyncClient>,
) -> impl IntoResponse {
    let resp = client
        .lock()
        .await
        .get_streams_all(params.category.game_id())
        .await;

    let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
            error!("failed querying twitch: {:?}", e);
            return templates::Results::error();
        }
    };

    let words = create_query_words(&params.query);
    let streams = filter_streams(resp, &words, &params.language, |s| s);

    templates::Results::new(words, streams)
}

fn create_query_words(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
}

fn filter_streams<T>(
    streams: Vec<Stream>,
    words: &[String],
    language: &str,
    map: impl Fn(Stream) -> T,
) -> Vec<T> {
    streams
        .into_iter()
        .filter_map(|stream| {
            let title = stream.title.to_lowercase();

            let is_match = words.iter().any(|w| title.contains(w))
                && (language.is_empty() || stream.language == language);

            is_match.then(|| map(stream))
        })
        .collect()
}

pub async fn error(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::load_shed::error::Overloaded>() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "service is overloaded, try again later",
        )
    } else if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "request timed out")
    } else {
        error!(?err, "unhandled error");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}
