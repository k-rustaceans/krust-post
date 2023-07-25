use axum::Extension;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

#[derive(Clone)]
pub struct HeaderMessage(pub String);

pub async fn read_middleware_custom_header(Extension(message): Extension<HeaderMessage>) -> String {
    message.0
}

pub async fn set_middleware_custom_header<B>(mut request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let message = headers
        .get("message")
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;

    let message = message.to_str().map_err(|_error| StatusCode::BAD_REQUEST)?.to_owned();

    let extensions = request.extensions_mut();
    extensions.insert(HeaderMessage(message));

    Ok(next.run(request).await)
}
