use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// 模块定义的错误码
#[derive(Error, Debug)]
pub enum Error {
    /// 用户未登录
    #[error("user not login")]
    UserNotLogin,
    /// 用户密码不对
    #[error("password not correct")]
    UserPasswdError,
    /// 用户不存在
    #[error("user not exists")]
    UserNoExists,
    /// 验证码
    #[error("captcha error")]
    Captcha,
    /// 用户存在
    #[error("{0}")]
    UserExists(String),
    /// 接口未实现异常
    #[error("Parse \"{0}\" error")]
    Parse(String),
    /// 解析json错误
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
    /// 接口未实现异常
    #[error("database exception")]
    DatabaseException,
    /// 自定义错误，为了偷懒，不定义太多错误。
    /// 未能上述表示的错误，一律用此表示
    #[error("Error: {0}")]
    Custom(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (error, message) = match self {
            Error::Custom(message) => (ERR_INTERNAL, message),
            Error::JsonExtractorRejection(reject) => (
                ERR_PROTOCOL,
                format!("protocol parse error: \"{}\"", reject.body_text()),
            ),
            Error::Parse(message) => (ERR_PARSE_REQ, format!("protocol error: \"{}\"", message)),
            Error::UserExists(message) => {
                (ERR_USER_EXISTS, format!("signup error: \"{}\"", message))
            }
            Error::DatabaseException => (ERR_INTERNAL, format!("database exception")),
            Error::Captcha => (ERR_CAPTCHA_ERROR, format!("captcha error")),
            Error::UserPasswdError => (ERR_USER_NOT_EXISTS, format!("user not exists")),
            Error::UserNoExists => (ERR_USER_PASSWD_ERROR, format!("user passwd error")),
            Error::UserNotLogin => (ERR_USER_NOT_LOGIN, format!("user not login")),
        };
        let body = Json(json!({
            "error": error,
            "message": message
        }));

        (StatusCode::OK, body).into_response()
    }
}

pub const ERR_SUCCESS: usize = 0;

pub const ERR_USER_NOT_LOGIN: usize = 300;

pub const ERR_PARSE_REQ: usize = 400;
pub const ERR_USER_EXISTS: usize = 401;
pub const ERR_CAPTCHA_ERROR: usize = 402;
pub const ERR_USER_NOT_EXISTS: usize = 403;
pub const ERR_USER_PASSWD_ERROR: usize = 404;

pub const ERR_INTERNAL: usize = 500;

pub const ERR_PROTOCOL: usize = 900;
