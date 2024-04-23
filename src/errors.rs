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
    UserNotExists,
    /// 产品不存在
    #[error("product not exists")]
    ProductNotExists,
    /// 产品不存在
    #[error("product not open")]
    ProductNotOpen,
    /// 验证码
    #[error("captcha error")]
    Captcha,
    /// 邮箱验证码
    #[error("captcha error")]
    Code,
    /// 操作过于频繁
    #[error("operation too frequent")]
    Frequent,
    /// 用户存在
    #[error("{0}")]
    UserExists(String),
    /// 密码过短
    #[error("password too illegal")]
    UserPassword,
    /// Token失败
    #[error("token create error")]
    TokenCreation,
    /// Token失败
    #[error("token invalid")]
    TokenInvalid,
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
            Error::Custom(message) => (E_INTERNAL, message),
            Error::JsonExtractorRejection(reject) => (
                E_BAD_PROTOCOL,
                format!("protocol parse error: \"{}\"", reject.body_text()),
            ),
            Error::Parse(message) => (E_PARSE_REQ, format!("protocol error: \"{}\"", message)),
            Error::UserExists(message) => (E_USER_EXISTS, format!("signup error: \"{}\"", message)),
            Error::DatabaseException => (E_INTERNAL, format!("database exception")),
            Error::Captcha => (E_BAD_CAPTCHA, format!("captcha error")),
            Error::UserPasswdError => (E_USER_NOT_EXISTS, format!("user password error")),
            Error::UserNotExists => (E_BAD_PASSWD, format!("user not exists")),
            Error::UserNotLogin => (E_USER_NOT_LOGIN, format!("user not login")),
            Error::Code => (E_BAD_CAPTCHA, format!("email captcha error")),
            Error::Frequent => (E_TOO_FREQUENT, format!("operation too frequent")),
            Error::ProductNotExists => (E_PRODUCT_NOT_EXISTS, format!("product not exists")),
            Error::ProductNotOpen => (E_PRODUCT_NOT_OPEN, format!("product not open")),
            Error::TokenCreation => (E_TOKEN_CREATION, format!("token creation error")),
            Error::TokenInvalid => (E_TOKEN_INVALID, format!("token invalid")),
            Error::UserPassword => (
                E_PASSWORD_ILLEGAL,
                format!("password illegal, length must at least 6"),
            ),
        };
        let body = Json(json!({
            "error": error,
            "message": message
        }));

        (StatusCode::OK, body).into_response()
    }
}

pub const E_SUCCESS: usize = 0;

pub const E_USER_NOT_LOGIN: usize = 300;
pub const E_PARSE_REQ: usize = 400;
pub const E_USER_EXISTS: usize = 401;
pub const E_BAD_CAPTCHA: usize = 402;
pub const E_USER_NOT_EXISTS: usize = 403;
pub const E_BAD_PASSWD: usize = 404;
pub const E_TOO_FREQUENT: usize = 405;
pub const E_PRODUCT_NOT_EXISTS: usize = 406;
pub const E_PRODUCT_NOT_OPEN: usize = 407;
pub const E_TOKEN_CREATION: usize = 408;
pub const E_TOKEN_INVALID: usize = 409;
pub const E_PASSWORD_ILLEGAL: usize = 410;

pub const E_INTERNAL: usize = 500;
pub const E_BAD_PROTOCOL: usize = 900;
