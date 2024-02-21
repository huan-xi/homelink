use std::fmt::Debug;
use axum::body::{Body, BoxBody};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use crate::api::errors::ApiError;
use crate::api::json::to_json_string;

pub type ApiResult<T> = Result<ApiResp<T>, ApiError>;

#[macro_export]
macro_rules! api_err {
    () => {
        crate::api::errors::ApiError::msg("未知错误")
    };
    ($($arg:tt)*) => {{
        crate::api::errors::ApiError::msg(format_args!($($arg)*).to_string())
    }};
}

#[macro_export]
macro_rules! err_msg {
    ($($arg:tt)*) => {{
         Err(crate::api_err!($($arg)*))
    }};
}

#[derive(Debug, Serialize)]
pub struct ApiResp<T> {
    pub code: i32,
    pub data: Option<T>,
    pub total: Option<u64>,
    pub msg: String,
}


/// 填入到extensions中的数据
#[derive(Debug, Clone)]
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
                let box_body = axum::body::boxed(body);
                // let body = Body::from(e.to_string());
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(box_body)
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

pub fn err_msg_string<T>(str: String) -> ApiResult<T> {
    Err(ApiError::Msg(str))
}

pub fn output_err_msg<T>(str: String) -> ApiResult<T> {
    Err(ApiError::Msg(str))
}

pub fn err_msg<T>(str: &'static str) -> ApiResult<T> {
    Err(ApiError::StrMsg(str))
}



pub fn ok_data<T>(data: T) -> ApiResult<T> {
    Ok(ApiResp {
        code: 200,
        data: Some(data),
        total: None,
        msg: "success".to_string(),
    })
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