use axum::Json;
use once_cell::sync::Lazy;

pub mod app_router;
pub mod app_state;
pub mod config;
pub mod errors;
pub mod handler;
pub mod model;
pub mod repo;
pub mod proto;
pub mod util;

/// 模块定义结果状态
pub type Result<T = ()> = std::result::Result<T, crate::errors::Error>;
pub type JsonResult<T> = std::result::Result<Json<T>, crate::errors::Error>;
pub type JsonRejection<T> = axum_extra::extract::WithRejection<Json<T>, errors::Error>;

pub static CONFIG: Lazy<config::Config> = Lazy::new(config::Config::load_config);
