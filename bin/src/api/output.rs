use std::fmt::Debug;
use axum::body;
use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use crate::api::errors::ApiError;
use crate::api::json_1::to_json_string;

pub type ApiResult<T> = Result<ApiResp<T>, ApiError>;

#[derive(Debug, Serialize)]
pub struct ApiResp<T> {
    pub code: i32,
    pub data: Option<T>,
    pub total: Option<u64>,
    pub msg: String,
}


/// 填入到extensions中的数据
#[derive(Debug,Clone)]
pub struct ResJsonString(pub String);


#[allow(unconditional_recursion)]
impl<T> IntoResponse for ApiResp<T>
    where
        T: Serialize + Send + Sync + Debug + 'static,
{
    fn into_response(self) -> Response {
        let data = Self {
            code: self.code,
            data: self.data,
            total: self.total,
            msg: self.msg,
        };
        //to string
        let json_string = match to_json_string(&data) {
            Ok(v) => v,
            Err(e) => {
                let body = Body::from(e.to_string());
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(body)
                    .unwrap();
            }
        };
        let res_json_string = ResJsonString(json_string.clone());

        let mut response = json_string.into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response.extensions_mut().insert(res_json_string);
        response
    }
}

impl ApiResp<()> {
    pub fn with_err(err: &str) -> Self {
        Self {
            code: -1,
            data: None,
            total: None,
            msg: err.to_string(),
        }
    }
    pub fn with_msg(msg: &str) -> Self {
        Self {
            code: 200,
            data: None,
            total: None,
            msg: msg.to_string(),
        }
    }
}

impl<T: Serialize> ApiResp<T> {
    pub fn with_list_data(data: T, total: u64) -> Self {
        Self {
            code: 200,
            data: Some(data),
            total: Some(total),
            msg: "success".to_string(),
        }
    }

    pub fn with_data(data: T) -> Self {
        Self {
            code: 200,
            data: Some(data),
            total: None,
            msg: "success".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_data_msg(data: T, msg: &str) -> Self {
        Self {
            code: 200,
            data: Some(data),
            total: None,
            msg: msg.to_string(),
        }
    }
}