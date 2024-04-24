use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    errors::E_SUCCESS,
    handler::ok_with_trace,
    proto::{GroupsReq, GroupsRsp},
    JsonRejection, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn groups(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload), _): JsonRejection<GroupsReq>,
) -> JsonResult<GroupsRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);

    let user_product = auth_user.user_product;
    let mut groups = state.repo.query_groups(user_product.user_id).await?;
    if let Some(payload) = payload.groups {
        if !payload.is_empty() {
            groups = groups
                .into_iter()
                .filter(|group| payload.contains(&group.name))
                .collect();
        }
    }
    let rsp = GroupsRsp {
        error: E_SUCCESS,
        message: "success".into(),
        groups,
    };

    ok_with_trace(rsp)
}
