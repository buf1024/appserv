use std::time::Duration;

use axum::{
    debug_handler,
    extract::State,
    headers::Cookie,
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
    Json, TypedHeader,
};
use axum_extra::extract::WithRejection;

use crate::{
    app_state::AppState,
    errors::{Error, ERR_SUCCESS},
    handler::COOKIE_NAME,
    proto::{SignInReq, SignInResp},
    JsonRejection, Result,
};
use async_session::{Session, SessionStore};

#[debug_handler(state = AppState)]
pub async fn signin(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    WithRejection(Json(payload), _): JsonRejection<SignInReq>,
) -> Result<impl IntoResponse> {
    {
        if payload.user_name.is_empty()
            || payload.passwd.is_empty()
            || payload.captcha_id.is_empty()
            || payload.captcha.is_empty()
        {
            return Err(Error::Parse(String::from(
                "some field is empty, please check.",
            )));
        }

        let cookie = cookies.get(COOKIE_NAME).ok_or(Error::Captcha)?;

        let session = state
            .store
            .load_session(cookie.to_string())
            .await
            .map_err(|_| Error::Captcha)?;
        if session.is_none() {
            return Err(Error::Captcha);
        }
        let session = session.unwrap();

        let captcha: String = session.get(&payload.captcha_id).ok_or(Error::Captcha)?;

        state
            .store
            .destroy_session(session)
            .await
            .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }
    }

    let user = state.repo.signin_user(payload).await?;
    tracing::info!("user log in: {:?}", &user);

    let headers = {
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(60 * 5));
        session
            .insert("user", user)
            .map_err(|e| Error::Custom(format!("store session error: {}", e)))?;

        let cookie = state
            .store
            .store_session(session)
            .await
            .map_err(|e| Error::Custom(format!("store session error: {}", e)))?
            .ok_or(Error::Custom(format!("store session error")))?;

        let cookie = format!("{}={cookie}; SameSite=Lax; Path=/", COOKIE_NAME);

        // Set cookie
        let mut headers = HeaderMap::new();
        headers.insert(
            SET_COOKIE,
            cookie
                .parse()
                .map_err(|e| Error::Custom(format!("failed to set header: {}", e)))?,
        );
        headers
    };
    let resp = SignInResp {
        error: ERR_SUCCESS,
        message: "success".to_string(),
    };

    Ok((headers, Json(resp)))
}
