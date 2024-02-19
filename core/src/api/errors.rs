use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::error;
use sea_orm::DbErr;
use strum_macros::Display;
use thiserror::Error;
use miot_proto::device::miot_spec_device::NotSupportMiotDeviceError;
use crate::api::output::ApiResp;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("DbErr")]
    DbErr(DbErr),
    #[error("{0}")]
    Msg(String),
    #[error("{0}")]
    StrMsg(&'static str),
    #[error("json 序列化错误")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl From<btleplug::Error> for ApiError {
    fn from(value: btleplug::Error) -> Self {
        ApiError::Msg(format!("btleplug error: {}", value))
    }
}

impl From<NotSupportMiotDeviceError> for ApiError {
    fn from(value: NotSupportMiotDeviceError) -> Self {
        ApiError::Msg("该设备不是米家设备类型".to_string())
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

impl From<DbErr> for ApiError {
    fn from(d: DbErr) -> Self {
        ApiError::DbErr(d)
    }
}
