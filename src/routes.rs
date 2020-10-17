use std::convert::Infallible;
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

use crate::twitch::{Category, TwitchClient};

type Client = Arc<Mutex<TwitchClient>>;

#[derive(Deserialize)]
struct SearchParams {
    category: Category,
    query: String,
    language: String,
}

pub(super) mod filters {
    use warp::{Filter, Rejection, Reply};

    use super::handlers;
    use super::{with_client, Client, SearchParams};

    /// GET /
    pub(super) fn index() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get().map(handlers::index)
    }

    /// GET /?category=ScienceAndTechnology&query=rust
    pub(super) fn search(
        client: Client,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(warp::query::<SearchParams>())
            .and(with_client(client))
            .and_then(handlers::search)
    }
}

pub(super) mod handlers {
    use std::convert::Infallible;

    use log::error;
    use warp::Reply;

    use super::{Client, SearchParams};
    use crate::templates;

    pub(super) fn index() -> impl Reply {
        templates::Index
    }

    pub(super) async fn search(
        params: SearchParams,
        client: Client,
    ) -> Result<impl Reply, Infallible> {
        let resp = client
            .lock()
            .await
            .get_streams_all(params.category.game_id())
            .await;

        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                error!("failed querying twitch: {:?}", e);
                return Ok(templates::Results::error());
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

        Ok(templates::Results::new(query_words, streams))
    }
}

pub fn build(client: Client) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    filters::search(client)
        .or(filters::index())
        .with(warp::log("api"))
        .with(warp::compression::gzip())
}

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}
