use axum::{debug_handler, extract::State};

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, handler::ok_with_trace,
    proto::ProductsRsp, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn user_products(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> JsonResult<ProductsRsp> {
    let user = auth_user.user;
    let products = state.repo.query_user_products(user.id.unwrap()).await?;
    let rsp = ProductsRsp {
        error: E_SUCCESS,
        message: "success".into(),
        products: products,
    };

    ok_with_trace(rsp)
}
