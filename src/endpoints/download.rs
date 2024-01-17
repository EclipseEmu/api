use {
    crate::{
        util::{empty_string_as_none, is_global_ip_url},
        ApiError, ApiState,
    },
    axum::{
        body::Body,
        extract::{Query, State},
        http::{HeaderName, HeaderValue, StatusCode},
        response::{IntoResponse, Response},
    },
    serde::Deserialize,
    std::str::FromStr,
};

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

    let req = http.get(parsed_url.to_string()).build()?;
    if !is_global_ip_url(req.url()) {
        return Err(ApiError::MissingQuery("invalid url"));
    }

    let resp = http.execute(req).await.map_err(|e| ApiError::Reqwest(e))?;

    let axum_status_code =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_REQUEST);
    let mut builder = Response::builder().status(axum_status_code);

    // Set the headers
    for (header, value) in resp.headers().iter() {
        let (Ok(axum_name), Ok(axum_value)) = (
            HeaderName::from_str(header.as_str()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) else {
            continue;
        };
        builder = builder.header(axum_name, axum_value);
    }

    // Make the stream
    let stream = Body::from_stream(resp.bytes_stream());
    match builder.body(stream) {
        Ok(body) => Ok(body),
        Err(err) => Err(ApiError::AxumHttp(err)),
    }
}
