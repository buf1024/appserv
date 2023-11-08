#![allow(unused_imports)]
use axum::{debug_handler, Json};

use crate::{
    app_state::AppState, errors::ERR_SUCCESS, model::user::User, proto::IsSignInResp, JsonResult,
};

#[debug_handler(state = AppState)]
pub async fn is_signin(user: Option<User>) -> JsonResult<IsSignInResp> {
    let is_signin = if user.is_some() { 1 } else { 0 };

    let resp = IsSignInResp {
        error: ERR_SUCCESS,
        is_signin,
    };
    Ok(Json(resp))
}
