use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use impl_new::New;
use log::error;
use sea_orm::DbErr;
use strum_macros::Display;
use thiserror::Error;
use miot_proto::device::miot_spec_device::NotSupportMiotDeviceError;
use crate::api::output::ApiResp;

#[derive(Debug,New)]
pub struct ApiErrorInner {
    pub code: i32,
    pub msg: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("strum error {0}")]
    ParseError(#[from] strum::ParseError),
    #[error("DbErr {0}")]
    DbErr(#[from] DbErr),
    #[error("{0}")]
    Msg(String),
    #[error("{0}")]
    StrMsg(&'static str),
    #[error("json 序列化错误")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    MiotDeviceError(NotSupportMiotDeviceError),
    #[error("btleplug error {0}")]
    BtlePlugError(#[from] btleplug::Error),
    #[error("{0}")]
    BadRequest(String),
    #[error("ApiErrorInner")]
    ApiErrorInner(ApiErrorInner),
}

impl From<NotSupportMiotDeviceError> for ApiError {
    fn from(value: NotSupportMiotDeviceError) -> Self {
        ApiError::MiotDeviceError(value)
    }
}

impl ApiError {
    pub fn msg(msg: String) -> Self {
        ApiError::Msg(msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Msg(msg) => ApiResp::<()>::with_err(msg.as_str()).into_response(),
            ApiError::StrMsg(msg) => ApiResp::<()>::with_err(msg).into_response(),
            ApiError::BadRequest(msg) => ApiResp::<()>::with_err(msg.as_str()).into_response(),
            ApiError::ApiErrorInner(inner) => {
                ApiResp::<()>::with_code(inner.code,inner.msg).into_response()
            }
            ApiError::DbErr(err) => {
                match err {
                    DbErr::RecordNotUpdated => {
                        ApiResp::<()>::with_err("记录未更新,可能数据不存在").into_response()
                    }
                    _ => {
                        error!("数据库错误:{:?}", err);
                        ApiResp::<()>::with_err("数据库错误").into_response()
                    }
                }
            }
            _ => {
                error!("服务器异常:{:?}", self);
                ApiResp::<()>::with_err("服务器异常").into_response()
            }
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: Error) -> Self {
        ApiError::Msg(value.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        ApiError::Msg("io error".to_string())
    }
}

