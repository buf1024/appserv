use std::time::Duration;

use axum::{debug_handler, extract::State, headers, Json, TypedHeader};
use axum_extra::extract::WithRejection;
use regex::Regex;

use crate::{
    app_state::AppState,
    errors::{Error, ERR_SUCCESS},
    handler::COOKIE_NAME,
    proto::{SignUpReq, SignUpResp},
    util, JsonRejection, JsonResult, CONFIG,
};
use async_session::{Session, SessionStore};

#[debug_handler(state = AppState)]
pub async fn signup(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    WithRejection(Json(payload), _): JsonRejection<SignUpReq>,
) -> JsonResult<SignUpResp> {
    {
        tracing::info!(?payload);
        if payload.user_name.is_empty()
            || payload.email.is_empty()
            || payload.passwd.is_empty()
            || payload.captcha_id.is_empty()
            || payload.captcha.is_empty()
        {
            return Err(Error::Parse(String::from(
                "some field is empty, please check.",
            )));
        }
        let re: Regex = Regex::new(r"(?x)^(?P<name>[^@\s]+)@([[:word:]]+\.)*[[:word:]]+$").unwrap();
        if re
            .captures(&payload.email)
            .and_then(|cap| cap.name("name").map(|name| name.as_str()))
            .is_none()
        {
            return Err(Error::Parse(String::from("email format is not correct.")));
        }
        let cookie = cookies
            .get(COOKIE_NAME)
            .ok_or(Error::Custom(format!("cookie not found in session")))?;

        let session = state
            .store
            .load_session(cookie.to_string())
            .await
            .map_err(|e| Error::Custom(format!("session not found: {}", e)))?;
        if session.is_none() {
            return Err(Error::Custom(format!(
                "session not found: {}",
                payload.captcha_id
            )));
        }
        let session = session.unwrap();

        let captcha: String = session
            .get(&payload.captcha_id)
            .ok_or(Error::Custom(format!("captcha session not found")))?;

        state
            .store
            .destroy_session(session)
            .await
            .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }
    }

    let user = state.repo.create_user(payload).await?;
    tracing::info!("new create user: {:?}", &user);

    {
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(60 * 30));
        session
            .insert("user_id", user.id.unwrap())
            .map_err(|e| Error::Custom(format!("store session error: {}", e)))?;

        let session_id = state
            .store
            .store_session(session)
            .await
            .map_err(|e| Error::Custom(format!("store session error: {}", e)))?
            .ok_or(Error::Custom(format!("store session error")))?;

        let active_id = urlencoding::encode(&session_id).to_string();

        tracing::debug!("active: {}", &active_id);

        tokio::spawn(async move {
            let subject = "激活链接/Activate link".to_string();
            let body = format!(
                "<h5>请点以下链接激活账户/Click the flowing link to activate account<h5><br><a>{}?activate={}</a>",
                CONFIG.activate_link.clone(), active_id);
            util::send_email(user.email, subject, body)
        });
    }
    let rsp = SignUpResp {
        error: ERR_SUCCESS,
        message: "success".to_string(),
    };

    Ok(Json(rsp))
}
