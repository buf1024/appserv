use axum::{debug_handler, extract::State};

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::BaseRsp, JsonResult,
};

#[debug_handler(state = AppState)]
pub async fn signout(State(state): State<AppState>, auth_user: AuthUser) -> JsonResult<BaseRsp> {
    state.repo.delete_session(&auth_user.token).await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
