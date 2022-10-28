use std::fmt::Write;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::header::CONTENT_TYPE;
use time::OffsetDateTime;
use tracing::error;

use crate::twitch::Stream;

pub struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(body) => ([(CONTENT_TYPE, T::MIME_TYPE)], body).into_response(),
            Err(err) => {
                error!(?err, "failed rendering template");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl<T> From<T> for HtmlTemplate<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "api_info.html")]
pub struct ApiInfo;

#[derive(Template)]
#[template(path = "results.html")]
pub struct Results {
    query_words: Vec<String>,
    streams: Vec<Stream>,
    error: bool,
}

impl Results {
    pub fn new(query_words: Vec<String>, streams: Vec<Stream>) -> Self {
        Self {
            query_words,
            streams,
            error: false,
        }
    }

    pub const fn error() -> Self {
        Self {
            query_words: Vec::new(),
            streams: Vec::new(),
            error: true,
        }
    }
}

fn since_now(value: &Option<OffsetDateTime>) -> String {
    if let Some(value) = value {
        let duration = OffsetDateTime::now_utc() - *value;
        let mut buf = String::new();

        match duration.whole_days() {
            0 => {}
            1 => buf.push_str("1 day"),
            d => write!(buf, "{d} days").unwrap(),
        }

        match duration.whole_hours() % 24 {
            0 => {}
            1 => buf.push_str(" 1 hour"),
            h => write!(buf, " {h} hours").unwrap(),
        }

        match duration.whole_minutes() % 60 {
            0 => {}
            1 => buf.push_str(" 1 minute"),
            m => write!(buf, " {m} minutes").unwrap(),
        }

        buf
    } else {
        String::new()
    }
}

#[allow(clippy::pedantic)]
fn sized(value: &str, width: u32, height: u32) -> String {
    value
        .replace("{width}", &width.to_string())
        .replace("{height}", &height.to_string())
}
