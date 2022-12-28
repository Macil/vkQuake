use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug)]
pub enum QuakeAPIResponseError {
    Err(Box<dyn std::error::Error>),
    #[allow(dead_code)]
    ErrResponse(Response),
}

impl<E: std::error::Error + 'static> From<E> for QuakeAPIResponseError {
    fn from(e: E) -> Self {
        Self::Err(e.into())
    }
}

impl IntoResponse for QuakeAPIResponseError {
    fn into_response(self) -> Response {
        match self {
            Self::Err(ref error) => {
                tracing::error!("Error in response Handler: {}", error.as_ref());
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
            Self::ErrResponse(response) => response,
        }
    }
}
