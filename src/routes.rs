use std::sync::Arc;

use axum::{
    routing::{get, post},
    AddExtensionLayer, Router,
};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::twitch::TwitchClient;

type Client = Arc<Mutex<TwitchClient>>;

pub fn build(client: Client) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new().route("/search", post(handlers::api::search)),
        )
        .route("/favicon-16x16.png", get(handlers::favicon_16))
        .route("/favicon-32x32.png", get(handlers::favicon_32))
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

pub mod handlers {
    #![allow(clippy::unused_async)]

    use axum::{
        body::Body,
        extract::{Extension, Query},
        http::Response,
        response::IntoResponse,
    };
    use serde::Deserialize;
    use tracing::error;

    use super::{responses::HtmlTemplate, Client};
    use crate::{
        templates,
        twitch::{Category, Stream},
    };

    #[derive(Deserialize)]
    pub struct SearchParams {
        category: Category,
        query: String,
        #[serde(default)]
        language: String,
    }

    pub async fn index() -> impl IntoResponse {
        HtmlTemplate(templates::Index)
    }

    pub async fn favicon_32() -> impl IntoResponse {
        Response::new(Body::from(
            &include_bytes!("../assets/favicon-32x32.png")[..],
        ))
    }

    pub async fn favicon_16() -> impl IntoResponse {
        Response::new(Body::from(
            &include_bytes!("../assets/favicon-16x16.png")[..],
        ))
    }

    pub async fn search(
        Query(params): Query<SearchParams>,
        Extension(client): Extension<Client>,
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
                return HtmlTemplate(templates::Results::error());
            }
        };

        let words = create_query_words(&params.query);
        let streams = filter_streams(resp, &words, &params.language, |s| s);

        HtmlTemplate(templates::Results::new(words, streams))
    }

    pub mod api {
        use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
        use serde::Serialize;
        use time::OffsetDateTime;
        use tracing::error;

        use super::{create_query_words, filter_streams, SearchParams};
        use crate::{lang, routes::Client, twitch::Stream};

        #[derive(Serialize)]
        struct Response {
            title: String,
            username: String,
            language: String,
            stream_time: Option<i64>,
            viewer_count: u64,
        }

        impl From<Stream> for Response {
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
            Json(params): Json<SearchParams>,
            Extension(client): Extension<Client>,
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

            let words = create_query_words(&params.query);
            let streams = filter_streams(resp, &words, &params.language, Response::from);

            Ok(Json(streams))
        }
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
}

pub mod responses {
    use askama::Template;
    use axum::{
        body::{self, BoxBody, Empty},
        http::{Response, StatusCode},
        response::{self, IntoResponse},
    };
    use tracing::error;

    pub struct HtmlTemplate<T>(pub T);

    impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
    {
        fn into_response(self) -> Response<BoxBody> {
            match self.0.render() {
                Ok(html) => response::Html(html).into_response(),
                Err(err) => {
                    error!(?err, "failed rendering template");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(body::boxed(Empty::default()))
                        .unwrap()
                }
            }
        }
    }
}
