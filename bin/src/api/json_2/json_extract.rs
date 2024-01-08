use axum::{async_trait};
use axum::extract::rejection::{JsonRejection, JsonSyntaxError};
use axum::response::{IntoResponse, Response};
use axum::{
    body::{Bytes, HttpBody},
    extract::{rejection::*, FromRequest},
    BoxError,
};
// use async_trait::async_trait;
// use axum_core::response::{IntoResponse, Response};
use bytes::{BufMut, BytesMut};
use axum::http::{
    header::{self, HeaderMap, HeaderValue},
    Request, StatusCode,
};
use log::info;
use serde::{de::DeserializeOwned, Serialize};



#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "json_2")))]
#[must_use]
pub struct Json<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for Json<T>
    where
        T: DeserializeOwned,
        B: HttpBody + Send + 'static,
        B::Data: Send,
        B::Error: Into<BoxError>,
        S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        if json_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);

            let value = match serde_path_to_error::deserialize(deserializer) {
                Ok(value) => value,
                Err(err) => {
                    info!("json_2 序列化错误 err:{:?}", err);
                    todo!()
                   /* let rejection = match err.inner().classify() {
                        serde_json::error::Category::Data =>
                            JsonDataError::from_err(err).into(),
                        serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                            JsonSyntaxError::from_err(err).into()
                        }
                        serde_json::error::Category::Io => {
                            if cfg!(debug_assertions) {
                                // we don't use `serde_json::from_reader` and instead always buffer
                                // bodies first, so we shouldn't encounter any IO errors
                                unreachable!()
                            } else {
                                JsonSyntaxError::from_err(err).into()
                            }
                        }
                    };
                    return Err(rejection);*/
                }
            };

            Ok(Json(value))
        } else {
            todo!();
            // Err(MissingJsonContentType.into())
        }
    }
}

fn json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json_2" || mime.suffix().map_or(false, |name| name == "json_2"));

    is_json_content_type
}

// axum_core::__impl_deref!(Json);

impl<T> From<T> for Json<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Json<T>
    where
        T: Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
