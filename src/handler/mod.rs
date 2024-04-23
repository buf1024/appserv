use std::time::Duration;

use async_session::{MemoryStore, SessionStore};
use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
    RequestPartsExt,
};
use axum_extra::{headers, typed_header::TypedHeaderRejectionReason, TypedHeader};

mod captcha;
pub use captcha::captcha;

mod send_email_code;
pub use send_email_code::send_email_code;

mod user;
pub use user::*;

use crate::{errors::Error, model::user::User};

pub const COOKIE_NAME: &'static str = "SESSION";
