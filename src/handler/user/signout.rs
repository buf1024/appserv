use async_session::SessionStore;
use axum::{debug_handler, extract::State, Json};
use axum_extra::{headers::Cookie, TypedHeader};

use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS, E_USER_NOT_LOGIN},
    handler::COOKIE_NAME,
    model::user::User,
    proto::SignOutResp,
    JsonResult,
};

#[debug_handler(state = AppState)]
pub async fn signout(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> JsonResult<SignOutResp> {
    let is_login = {
        let mut is_login = false;
        let cookie = cookies.get(COOKIE_NAME);

        if let Some(cookie) = cookie {
            let session = state
                .store
                .load_session(cookie.to_string())
                .await
                .map_err(|e| Error::Custom(format!("session not found: {}", e)))?;
            if let Some(session) = session {
                if let Some(_) = session.get::<User>("user") {
                    is_login = true;
                    state
                        .store
                        .destroy_session(session)
                        .await
                        .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;
                }
            }
        }
        is_login
    };

    let resp = if is_login {
        SignOutResp {
            error: E_SUCCESS,
            message: "success".to_string(),
        }
    } else {
        SignOutResp {
            error: E_USER_NOT_LOGIN,
            message: "user not login".to_string(),
        }
    };

    Ok(Json(resp))
}
