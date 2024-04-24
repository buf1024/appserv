use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::FavoritesRsp, JsonResult,
};
use axum::{debug_handler, extract::State};
#[debug_handler(state = AppState)]
pub async fn favorites(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> JsonResult<FavoritesRsp> {
    let user_product = auth_user.user_product;
    let favorites = state.repo.query_favorites(user_product.user_id).await?;

    let rsp = FavoritesRsp {
        error: E_SUCCESS,
        message: "success".into(),
        favorites,
    };

    ok_with_trace(rsp)
}
