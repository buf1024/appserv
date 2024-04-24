use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState, auth_user::AuthUser, errors::{Error, E_SUCCESS}, handler::ok_with_trace, proto::{BaseRsp, ModifyReq}, JsonRejection, JsonResult
};
#[debug_handler(state = AppState)]
pub async fn modify(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload), _): JsonRejection<ModifyReq>,
) -> JsonResult<BaseRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);

    if let Some(password) = &payload.password {
        if password.len() < 6 {
            return Err(Error::UserPassword);
        }
    }
    let user_product = auth_user.user_product;
    state
        .repo
        .update_user_info(
            user_product.user_id,
            user_product.product_id,
            payload.user_name,
            payload.password,
            payload.avatar_path,
        )
        .await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
