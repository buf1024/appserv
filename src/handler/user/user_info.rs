#![allow(unused_imports)]
use axum::{debug_handler, Json};

use crate::{
    app_state::AppState, errors::E_SUCCESS, model::user::User, proto::UserInfoResp, JsonResult,
};

#[debug_handler(state = AppState)]
pub async fn user_info(user: User) -> JsonResult<UserInfoResp> {
    let resp = UserInfoResp {
        error: E_SUCCESS,
        user_name: user.user_name,
        email: user.email,
        avatar: user.avatar,
    };

    Ok(Json(resp))
}
