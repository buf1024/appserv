use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    errors::E_SUCCESS,
    handler::ok_with_trace,
    proto::{BaseRsp, RecentlyModifyReq},
    JsonRejection, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn recently_modify(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload), _): JsonRejection<RecentlyModifyReq>,
) -> JsonResult<BaseRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);

    let user_product = auth_user.user_product;
    state
        .repo
        .modify_recently(
            user_product.user_id,
            &payload.stationuuid,
            payload.start_time,
            payload.end_time,
        )
        .await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
