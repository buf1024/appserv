use std::time::Duration;

use async_session::{Session, SessionStore};
use axum::{
    debug_handler,
    extract::State,
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
    Json,
};
use axum_extra::{headers, TypedHeader};
use captcha::{filters::Noise, Captcha};

use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS},
    proto::CaptchaRsp,
    Result,
};

use super::COOKIE_NAME;

#[debug_handler(state = AppState)]
pub async fn captcha(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse> {
    {
        if let Some(cookie) = cookies.get(COOKIE_NAME) {
            if let Ok(Some(session)) = state.store.load_session(cookie.to_string()).await {
                let _ = state.store.destroy_session(session).await;
            }
        }
    }

    let (captcha, chars) = {
        let mut captcha = Captcha::new();
        captcha.add_chars(4);
        captcha.apply_filter(Noise::new(0.3));
        captcha.view(120, 52);
        let chars = captcha.chars_as_string();
        tracing::info!("captcha: {}", &chars);
        let captcha = captcha
            .as_base64()
            .ok_or(Error::Internal(String::from("fail to generate captcha")))?;

        (captcha, chars)
    };

    let mut session = Session::new();
    session.expire_in(Duration::from_secs(60 * 5));
    session
        .insert("captcha", chars)
        .map_err(|e| Error::Internal(format!("new session error: {}", e)))?;

    let cookie = state
        .store
        .store_session(session)
        .await
        .map_err(|e| Error::Internal(format!("store session error: {}", e)))?
        .ok_or(Error::Internal(format!("store session error")))?;

    let resp = CaptchaRsp {
        error: E_SUCCESS,
        captcha,
    };

    let cookie = format!("{}={cookie}; SameSite=Lax; Path=/", COOKIE_NAME);

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie
            .parse()
            .map_err(|e| Error::Internal(format!("failed to set header: {}", e)))?,
    );

    Ok((headers, Json(resp)))
}
