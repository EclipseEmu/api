use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum ApiError {
    Reqwest(reqwest::Error),
    Sqlx(sqlx::Error),
    AxumHttp(axum::http::Error),
    MissingQuery(&'static str),
    Utf8Error,
}

// Convert an `ApiError` into a axum response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::MissingQuery(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Utf8Error => {
                (StatusCode::BAD_REQUEST, "unable to parse the passed url").into_response()
            }
            ApiError::Reqwest(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
            ApiError::Sqlx(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
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

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

impl From<std::str::Utf8Error> for ApiError {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}
