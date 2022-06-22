use axum::{
    http::header::{ACCEPT_ENCODING, VARY},
    response::Response,
};

#[must_use]
pub fn add_vary_header(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    if !headers.contains_key(VARY) {
        headers.insert(VARY, ACCEPT_ENCODING.into());
    }
    resp
}
