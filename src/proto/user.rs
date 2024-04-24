use serde::{Deserialize, Serialize};

use crate::model::product::Product;

use super::BaseRsp;

/// 注册
#[derive(Debug, Deserialize)]
pub struct SignUpReq {
    pub product: String,
    pub email: String,
    pub passwd: String,
    pub captcha: String,
    pub code: String,
}

pub type SignUpRsp = BaseRsp;

/// 登录
#[derive(Debug, Deserialize)]
pub struct SignInReq {
    pub product: String,
    pub email: String,
    pub passwd: String,
    pub captcha: String,
    pub product_open_flag: bool,
}

#[derive(Debug, Serialize)]
pub struct SignInRsp {
    pub error: usize,
    pub message: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ProductsRsp {
    pub error: usize,
    pub message: String,
    pub products: Vec<Product>,
}

// 用户基本信息
#[derive(Debug, Serialize)]
pub struct UserInfoRsp {
    pub error: usize,
    pub message: String,
    pub user_name: String,
    pub email: String,
    pub avatar: Option<String>,
    pub product: String,
    pub product_desc: String,
}

// 上传文件
#[derive(Debug, Serialize)]
pub struct UploadRsp {
    pub error: usize,
    pub message: String,
    pub avatar_path: String,
}

/// 信息修改
#[derive(Debug, Deserialize)]
pub struct ModifyReq {
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub avatar_path: Option<String>,
}
