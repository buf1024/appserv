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
    #[error("verify code error")]
    EmailVerifyCode,
    /// 邮箱验证码
    #[error("email error")]
    EmailDiff,
    /// 邮箱验证码
    #[error("send email error")]
    SendEmail,
    /// 操作过于频繁
    #[error("operation too frequent")]
    Frequent,
    /// 用户存在
    #[error("{0}")]
    UserExists(String),
    /// 密码过短
    #[error("password too illegal")]
    UserPasswordTooShort,
    /// Token失败
    #[error("token invalid")]
    TokenInvalid,
    /// 接口未实现异常
    #[error("Parse \"{0}\" error")]
    Parse(String),
    /// 接口未实现异常
    #[error("Parse email error")]
    ParseEmail,
    /// 解析json错误
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
    /// 数据库异常
    #[error("database exception: \"{0}\" ")]
    DatabaseException(String),
    #[error("internal error: {0}")]
    Internal(String),
    /// 自定义错误，为了偷懒，不定义太多错误。
    /// 未能上述表示的错误，一律用此表示
    #[error("Error: {0}")]
    Custom(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (error, message) = match self {
            Error::Custom(message) => (E_CUSTOM, message),
            Error::JsonExtractorRejection(reject) => (
                E_BAD_PROTOCOL,
                format!("protocol parse error: \"{}\"", reject.body_text()),
            ),
            Error::Parse(message) => (E_PARSE_REQ, format!("protocol error: \"{}\"", message)),
            Error::ParseEmail => (E_PARSE_EMAIL_REQ, format!("protocol email error")),
            Error::SendEmail => (E_SEND_EMAIL, format!("send email error")),
            Error::UserExists(message) => (E_USER_EXISTS, format!("signup error: \"{}\"", message)),
            Error::DatabaseException(message) => {
                (E_DATABASE, format!("database exception: \"{}\"", message))
            }
            Error::Internal(message) => {
                (E_INTERNAL, format!("internal exception: \"{}\"", message))
            }
            Error::Captcha => (E_BAD_CAPTCHA, format!("captcha error")),
            Error::UserPasswdError => (E_BAD_PASSWD, format!("user password error")),
            Error::UserNotExists => (E_USER_NOT_EXISTS, format!("user not exists")),
            Error::UserNotLogin => (E_USER_NOT_LOGIN, format!("user not login")),
            Error::EmailVerifyCode => (E_EMAIL_VERIFY_CODE, format!("email captcha error")),
            Error::Frequent => (E_TOO_FREQUENT, format!("operation too frequent")),
            Error::ProductNotExists => (E_PRODUCT_NOT_EXISTS, format!("product not exists")),
            Error::ProductNotOpen => (E_PRODUCT_NOT_OPEN, format!("product not open")),
            Error::TokenInvalid => (E_TOKEN_INVALID, format!("token invalid")),
            Error::UserPasswordTooShort => (
                E_PASSWORD_TOOL_SHORT,
                format!("password illegal, length must at least 6"),
            ),
            Error::EmailDiff => (E_EMAIL_DIFF, format!("email not equal")),
        };
        let body = Json(json!({
            "error": error,
            "message": message
        }));

        tracing::info!("\nrsp: {:?}\n", body);

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
pub const E_SEND_EMAIL: usize = 407;
pub const E_TOKEN_INVALID: usize = 408;
pub const E_PASSWORD_TOOL_SHORT: usize = 409;
pub const E_EMAIL_DIFF: usize = 410;
pub const E_PARSE_EMAIL_REQ: usize = 411;
pub const E_EMAIL_VERIFY_CODE: usize = 412;

pub const E_INTERNAL: usize = 500;
pub const E_CUSTOM: usize = 501;
pub const E_DATABASE: usize = 502;
pub const E_PRODUCT_NOT_OPEN: usize = 888;
pub const E_BAD_PROTOCOL: usize = 900;
