use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS},
    handler::{ok_with_trace, COOKIE_NAME},
    proto::{BaseRsp, ResetPasswdReq},
    JsonRejection, JsonResult,
};
use async_session::SessionStore;
use axum::{debug_handler, extract::State, Json};
use axum_extra::{extract::WithRejection, headers, TypedHeader};

#[debug_handler(state = AppState)]
pub async fn reset_passwd(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    WithRejection(Json(payload), _): JsonRejection<ResetPasswdReq>,
) -> JsonResult<BaseRsp> {
    {
        tracing::info!("\nreq: {:?}\n", &payload);

        if payload.email.is_empty()
            || payload.passwd.is_empty()
            || payload.captcha.is_empty()
            || payload.code.is_empty()
        {
            return Err(Error::Parse(String::from(
                "some field is empty, please check.",
            )));
        }

        let cookie = cookies
            .get(COOKIE_NAME)
            .ok_or(Error::Custom(format!("cookie not found in session")))?;

        let session = state
            .store
            .load_session(cookie.to_string())
            .await
            .map_err(|e| Error::Custom(format!("session not found: {}", e)))?
            .ok_or(Error::Custom(format!("session not found")))?;

        let captcha: String = session
            .get("captcha")
            .ok_or(Error::Custom(format!("captcha not found")))?;

        let code: String = session
            .get("code")
            .ok_or(Error::Custom(format!("code not found")))?;

        let email: String = session
            .get("email")
            .ok_or(Error::Custom(format!("email not found")))?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }

        if code.to_lowercase() != payload.code.to_lowercase() {
            return Err(Error::Code);
        }

        if email != payload.email {
            return Err(Error::Email);
        }

        state
            .store
            .destroy_session(session)
            .await
            .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;
    }

    state.repo.reset_user_passwd(&payload).await?;

    let rsp = BaseRsp {
        error: E_SUCCESS,
        message: "success".to_string(),
    };

    ok_with_trace(rsp)
}
