use std::sync::Arc;

use axum::{handler::get, routing::BoxRoute, AddExtensionLayer, Router};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::twitch::TwitchClient;

type Client = Arc<Mutex<TwitchClient>>;

pub fn build(client: Client) -> Router<BoxRoute> {
    Router::new()
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
        .check_infallible()
        .boxed()
}

pub(super) mod handlers {
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
    use crate::{templates, twitch::Category};

    #[derive(Deserialize)]
    pub(super) struct SearchParams {
        category: Category,
        query: String,
        language: String,
    }

    pub(super) async fn index() -> impl IntoResponse {
        HtmlTemplate(templates::Index)
    }

    pub(super) async fn favicon_32() -> impl IntoResponse {
        Response::new(Body::from(
            &include_bytes!("../assets/favicon-32x32.png")[..],
        ))
    }

    pub(super) async fn favicon_16() -> impl IntoResponse {
        Response::new(Body::from(
            &include_bytes!("../assets/favicon-16x16.png")[..],
        ))
    }

    pub(super) async fn search(
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

        let query_words = params
            .query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();

        let streams = resp
            .into_iter()
            .filter(|r| {
                let title = r.title.to_lowercase();

                query_words.iter().any(|w| title.contains(w))
                    && (params.language.is_empty() || r.language == params.language)
            })
            .collect();

        HtmlTemplate(templates::Results::new(query_words, streams))
    }
}

pub mod responses {
    use std::convert::Infallible;

    use askama::Template;
    use axum::{
        body::{Bytes, Full},
        http::{Response, StatusCode},
        response::{self, IntoResponse},
    };

    pub struct HtmlTemplate<T>(pub T);

    impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
    {
        type Body = Full<Bytes>;
        type BodyError = Infallible;

        fn into_response(self) -> hyper::Response<Self::Body> {
            match self.0.render() {
                Ok(html) => response::Html(html).into_response(),
                Err(_) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::default())
                    .unwrap(),
            }
        }
    }
}
