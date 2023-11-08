use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct BaseResp {
    pub error: usize,
    pub message: String,
}

/// 验证码
#[derive(Debug, Serialize)]
pub struct CaptchaResp {
    pub error: usize,
    pub captcha_id: String,
    pub captcha: String,
}

/// 注册
#[derive(Debug, Deserialize)]
pub struct SignUpReq {
    pub user_name: String,
    pub email: String,
    pub passwd: String,
    pub captcha_id: String,
    pub captcha: String,
}

pub type SignUpResp = BaseResp;

/// 激活
#[derive(Debug, Deserialize)]
pub struct ActivateReq {
    pub activate: String,
}
pub type ActivateResp = BaseResp;

/// 登录
#[derive(Debug, Deserialize)]
pub struct SignInReq {
    pub user_name: String,
    pub passwd: String,
    pub captcha_id: String,
    pub captcha: String,
}

pub type SignInResp = BaseResp;

/// 登录
#[derive(Debug, Serialize)]
pub struct IsSignInResp {
    pub error: usize,
    pub is_signin: isize,
}

/// 用户基本信息
#[derive(Debug, Serialize)]
pub struct UserInfoResp {
    pub error: usize,
    pub user_name: String,
    pub email: String,
    pub avatar: Option<String>,
}

pub type SignOutResp = BaseResp;