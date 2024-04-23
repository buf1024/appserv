use axum::Json;

pub mod app_router;
pub mod app_state;
pub mod config;
pub mod errors;
pub mod handler;
pub mod model;
pub mod proto;
pub mod repo;
pub mod util;

pub mod jwt;

pub mod auth_user;

/// 模块定义结果状态
pub type Result<T = ()> = std::result::Result<T, crate::errors::Error>;
pub type JsonResult<T> = std::result::Result<Json<T>, crate::errors::Error>;
pub type JsonRejection<T> = axum_extra::extract::WithRejection<Json<T>, errors::Error>;


