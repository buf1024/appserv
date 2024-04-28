use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState,
    auth_user::AuthUser,
    errors::{Error, E_SUCCESS},
    handler::ok_with_trace,
    proto::{BaseRsp, ModifyReq},
    util::gen_passwd,
    JsonRejection, JsonResult,
};
#[debug_handler(state = AppState)]
pub async fn modify(
    State(state): State<AppState>,
    auth_user: AuthUser,
    WithRejection(Json(payload), _): JsonRejection<ModifyReq>,
) -> JsonResult<BaseRsp> {
    tracing::info!("\nreq: {:?}\n", &payload);

    let mut new_pass = None;
    if let Some(password) = &payload.new_password {
        if password.len() < 6 {
            return Err(Error::UserPassword);
        }
        if payload.password.is_none() {
            return Err(Error::UserPasswdError);
        }

        let user = auth_user.user;
        let old_pass = gen_passwd(&user.email, &payload.password.unwrap());
        let new_pass_tmp = gen_passwd(&user.email, &password);
        if old_pass != new_pass_tmp {
            return Err(Error::UserPasswdError);
        }
        new_pass = Some(new_pass_tmp);
    }

    let user_product = auth_user.user_product;

    state
        .repo
        .update_user_info(
            user_product.user_id,
            user_product.product_id,
            payload.user_name,
            new_pass,
            payload.avatar_path,
        )
        .await?;
    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".into(),
    };

    ok_with_trace(rsp)
}
