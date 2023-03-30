use crate::{
    util::{empty_string_as_none, ReqwestAxumStream},
    ApiError, ApiState,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    url: Option<String>,
}

pub async fn handle(
    Query(params): Query<Params>,
    State(ApiState { http, .. }): State<ApiState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(url) = params.url else {
        return Err(ApiError::MissingQuery("missing url param"));
    };
    let parsed_url = percent_encoding::percent_decode_str(url.as_str()).decode_utf8()?;
    tracing::debug!("requesting url: {parsed_url:?}");
    let req = http.get(parsed_url.to_string()).build()?;
    match http.execute(req).await {
        Ok(resp) => Ok(ReqwestAxumStream(resp)),
        Err(e) => Err(ApiError::Reqwest(e)),
    }
}
