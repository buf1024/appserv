use serde::{Deserialize, Serialize};

mod user;
pub use user::*;

mod hiqradio;
pub use hiqradio::*;


#[derive(Debug, Serialize)]
pub struct BaseRsp {
    pub error: usize,
    pub message: String,
}

/// 验证码
#[derive(Debug, Serialize)]
pub struct CaptchaRsp {
    pub error: usize,
    pub captcha: String,
}

/// email验证码
#[derive(Debug, Deserialize)]
pub struct SendEmailCodeReq {
    pub email: String,
    pub captcha: String,
}

pub type SendEmailCodeRsp = BaseRsp;
