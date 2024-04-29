use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::BaseRsp, JsonResult,
};
use axum::{debug_handler, extract::State};
#[debug_handler(state = AppState)]
pub async fn is_login(State(_state): State<AppState>, _auth_user: AuthUser) -> JsonResult<BaseRsp> {
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
