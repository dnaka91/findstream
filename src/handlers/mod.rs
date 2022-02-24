#![allow(clippy::unused_async)]

pub mod api;

use axum::{
    body::Body,
    extract::{Extension, Query},
    http::Response,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::error;

use crate::{
    templates,
    twitch::{AsyncClient, Category, Stream},
};

#[derive(Deserialize)]
pub struct SearchParams {
    category: Category,
    query: String,
    #[serde(default)]
    language: String,
}

pub async fn index() -> impl IntoResponse {
    templates::Index
}

pub async fn api_info() -> impl IntoResponse {
    templates::ApiInfo
}

pub async fn favicon_32() -> impl IntoResponse {
    Response::new(Body::from(
        &include_bytes!("../../assets/favicon-32x32.png")[..],
    ))
}

pub async fn favicon_16() -> impl IntoResponse {
    Response::new(Body::from(
        &include_bytes!("../../assets/favicon-16x16.png")[..],
    ))
}

pub async fn search(
    Query(params): Query<SearchParams>,
    Extension(client): Extension<AsyncClient>,
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
