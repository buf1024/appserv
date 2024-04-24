#![allow(unused_imports)]
use std::{fs, path::Path};

use axum::{debug_handler, extract::State, Json};
use base64::prelude::*;

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    config::CONFIG,
    errors::{Error, E_SUCCESS},
    handler::ok_with_trace,
    model::{user::User, user_product},
    proto::UserInfoRsp,
    JsonResult,
};
use rand::{thread_rng, Rng};

#[debug_handler(state = AppState)]
pub async fn user_info(State(_): State<AppState>, auth_user: AuthUser) -> JsonResult<UserInfoRsp> {
    let user = auth_user.user;
    let product = auth_user.product;
    let user_product = auth_user.user_product;

    let mut avatar = None;
    if !user_product.avatar.is_empty() {
        let path = format!("{}/{}", &CONFIG.avatar_path, user_product.avatar);
        let path = Path::new(&path);
        if path.exists() && path.is_file() {
            let data = fs::read(path)
                .map_err(|e| Error::Custom(format!("read file error: {}", e.to_string())))?;

            let base64 = BASE64_STANDARD.encode(data);
            avatar = Some(base64);
        }
    }
    let rsp = UserInfoRsp {
        error: E_SUCCESS,
        message: "success".into(),
        user_name: user.user_name,
        email: user.email,
        avatar,
        product: product.product,
        product_desc: product.desc,
    };

    ok_with_trace(rsp)
}
