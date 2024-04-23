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
    pub new_token: Option<String>
}

// 登录
// #[derive(Debug, Serialize)]
// pub struct IsSignInRsp {
//     pub error: usize,
//     pub is_signin: isize,
// }

// 用户基本信息
// #[derive(Debug, Serialize)]
// pub struct UserInfoRsp {
//     pub error: usize,
//     pub user_name: String,
//     pub email: String,
//     pub avatar: Option<String>,
// }

// pub type SignOutRsp = BaseRsp;