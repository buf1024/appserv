use axum::{debug_handler, extract::State};

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::RecentlyRsp, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn recently(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> JsonResult<RecentlyRsp> {
    let user_product = auth_user.user_product;
    let recently = state.repo.query_recently(user_product.user_id).await?;
    let rsp = RecentlyRsp {
        error: E_SUCCESS,
        message: "success".into(),
        recently,
    };

    ok_with_trace(rsp)
}
