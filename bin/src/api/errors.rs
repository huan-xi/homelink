use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::error;
use sea_orm::DbErr;
use crate::api::output::ApiResp;

#[derive(Debug)]
pub enum ApiError {
    DbErr(DbErr),
    Msg(String),
    StrMsg(&'static str),
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

impl From<DbErr> for ApiError {
    fn from(d: DbErr) -> Self {
        ApiError::DbErr(d)
    }
}
