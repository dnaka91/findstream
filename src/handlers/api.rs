use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use time::OffsetDateTime;
use tracing::error;

use super::SearchParams;
use crate::{
    lang,
    twitch::{AsyncClient, Stream},
};

#[derive(Serialize)]
struct SimpleStream {
    title: String,
    username: String,
    language: String,
    stream_time: Option<i64>,
    viewer_count: u64,
}

impl From<Stream> for SimpleStream {
    fn from(stream: Stream) -> Self {
        Self {
            title: stream.title,
            username: stream.user_name,
            language: lang::translate_iso_639_1(&stream.language).to_owned(),
            stream_time: stream
                .started_at
                .map(|started_at| (OffsetDateTime::now_utc() - started_at).whole_seconds()),
            viewer_count: stream.viewer_count,
        }
    }
}

pub async fn search(
    State(client): State<AsyncClient>,
    Json(params): Json<SearchParams>,
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
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let words = super::create_query_words(&params.query);
    let streams = super::filter_streams(resp, &words, &params.language, SimpleStream::from);

    Ok(Json(streams))
}
