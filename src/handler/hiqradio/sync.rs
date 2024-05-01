use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    errors::E_SUCCESS,
    handler::ok_with_trace,
    proto::{SyncReq, SyncRsp},
    JsonRejection, JsonResult,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;
#[debug_handler(state = AppState)]
pub async fn sync(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload) , _): JsonRejection<SyncReq>,
) -> JsonResult<SyncRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);
    let user_product = auth_user.user_product;
    let (groups, recently, favorites) = state
        .repo
        .query_sync(user_product.user_id, -1)
        .await?;

    let rsp = SyncRsp {
        error: E_SUCCESS,
        message: "success".into(),
        groups,
        recently,
        favorites,
    };

    ok_with_trace(rsp)
}
