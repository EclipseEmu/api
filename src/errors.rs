use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

pub enum ApiError {
    Reqwest(reqwest::Error),
    AxumHttp(axum::http::Error),
    MissingQuery(&'static str),
    UrlParseError,
    Utf8Error,
}

// Convert an `ApiError` into a axum response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::MissingQuery(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Utf8Error | ApiError::UrlParseError => {
                (StatusCode::BAD_REQUEST, "unable to parse the passed url").into_response()
            }
            ApiError::Reqwest(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
            ApiError::AxumHttp(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<axum::http::Error> for ApiError {
    fn from(e: axum::http::Error) -> Self {
        Self::AxumHttp(e)
    }
}

impl From<url::ParseError> for ApiError {
    fn from(_: url::ParseError) -> Self {
        Self::UrlParseError
    }
}

impl From<std::str::Utf8Error> for ApiError {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}
