use axum::{debug_handler, extract::State};

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::BaseRsp, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn recently_clear(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> JsonResult<BaseRsp> {
    let user_product = auth_user.user_product;
    state.repo.delete_recently(user_product.user_id).await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
