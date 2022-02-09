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
