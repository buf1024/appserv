use std::time::Duration;

use async_session::{Session, SessionStore};
use axum::{
    debug_handler,
    extract::State,
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
    Json,
};
use captcha::{by_name, CaptchaName, Difficulty};
use rand::{thread_rng, Rng};

use crate::{
    app_state::AppState,
    errors::{Error, ERR_SUCCESS},
    proto::CaptchaResp,
    Result,
};

use super::COOKIE_NAME;

#[debug_handler(state = AppState)]
pub async fn captcha(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let (captcha_id, captcha, chars) = {
        let mut rng = thread_rng();
        let difficulty = rng.gen_range(0..3);
        let difficulty = match difficulty {
            0 => Difficulty::Easy,
            1 => Difficulty::Medium,
            2 => Difficulty::Hard,
            _ => unreachable!(),
        };

        let name = rng.gen_range(0..3);
        let name = match name {
            0 => CaptchaName::Amelia,
            1 => CaptchaName::Lucy,
            2 => CaptchaName::Mila,
            _ => unreachable!(),
        };

        let captcha_id = nanoid::nanoid!();
        let captcha = by_name(difficulty, name);
        let chars = captcha.chars_as_string();
        tracing::info!("captcha: {}", &chars);
        let captcha = captcha
            .as_base64()
            .ok_or(Error::Custom(String::from("fail to generate captcha")))?;

        (captcha_id, captcha, chars)
    };

    let mut session = Session::new();
    session.expire_in(Duration::from_secs(60 * 5));
    session
        .insert(&captcha_id, chars)
        .map_err(|e| Error::Custom(format!("new session error: {}", e)))?;

    let cookie = state
        .store
        .store_session(session)
        .await
        .map_err(|e| Error::Custom(format!("store session error: {}", e)))?
        .ok_or(Error::Custom(format!("store session error")))?;

    let resp = CaptchaResp {
        error: ERR_SUCCESS,
        captcha_id,
        captcha,
    };

    let cookie = format!("{}={cookie}; SameSite=Lax; Path=/", COOKIE_NAME);

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie
            .parse()
            .map_err(|e| Error::Custom(format!("failed to set header: {}", e)))?,
    );

    Ok((headers, Json(resp)))
}
