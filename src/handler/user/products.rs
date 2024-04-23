use axum::{extract::State, Json};

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::E_SUCCESS, proto::ProductsRsp, JsonResult,
};

pub async fn products(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> JsonResult<ProductsRsp> {
    let user = auth_user.user;
    let products = state.repo.query_products(user.user_id).await?;
    let resp = ProductsRsp {
        error: E_SUCCESS,
        message: "success".into(),
        products: products,
        new_token: auth_user.new_token,
    };

    Ok(Json(resp))
}
