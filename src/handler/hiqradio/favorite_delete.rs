use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    errors::{Error, E_SUCCESS},
    handler::ok_with_trace,
    proto::{BaseRsp, FavoriteDeleteReq},
    JsonRejection, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn favorite_delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload), _): JsonRejection<FavoriteDeleteReq>,
) -> JsonResult<BaseRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);

    if payload.favorites.is_none() && payload.group_names.is_none() {
        return Err(Error::Parse(String::from(
            "favorites or group is both none, please check",
        )));
    }

    let user_product = auth_user.user_product;
    state
        .repo
        .delete_favorite(
            user_product.user_id,
            &payload.favorites,
            &payload.group_names,
        )
        .await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
