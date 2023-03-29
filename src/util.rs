use crate::ApiError;
use axum::{
    body::{boxed, StreamBody},
    response::{IntoResponse, Response},
};
use serde::{de, Deserialize, Deserializer};
use std::{fmt, str::FromStr};

/// Serde deserialization decorator to map empty Strings to None,
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

// Convert a `reqwest::Response` into an axum response
pub struct ReqwestAxumStream(pub reqwest::Response);

impl IntoResponse for ReqwestAxumStream {
    fn into_response(self) -> axum::response::Response {
        let ReqwestAxumStream(resp) = self;
        let mut builder = Response::builder().status(resp.status());

        // Set the headers
        for (header, value) in resp.headers().iter() {
            builder = builder.header(header, value.to_owned());
        }

        // Make the stream
        let stream = StreamBody::new(resp.bytes_stream());
        match builder.body(boxed(stream)) {
            Ok(body) => body,
            Err(err) => ApiError::AxumHttp(err).into_response(),
        }
    }
}
