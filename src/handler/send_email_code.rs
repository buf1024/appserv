use std::time::Duration;

use axum::{debug_handler, extract::State, Json};
use axum_extra::{extract::WithRejection, headers, TypedHeader};
use chrono::Local;
use rand::{thread_rng, Rng};
use regex::Regex;

use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS},
    handler::COOKIE_NAME,
    proto::{SendEmailCodeReq, SendEmailCodeRsp},
    util, JsonRejection, JsonResult,
};
use async_session::SessionStore;

#[debug_handler(state = AppState)]
pub async fn send_email_code(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    WithRejection(Json(payload), _): JsonRejection<SendEmailCodeReq>,
) -> JsonResult<SendEmailCodeRsp> {
    let mut session = {
        tracing::info!(?payload);
        if payload.email.is_empty() || payload.captcha.is_empty() {
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
            .map_err(|e| Error::Custom(format!("session not found: {}", e)))?
            .ok_or(Error::Custom(format!("session not found")))?;

        let captcha: String = session
            .get("captcha")
            .ok_or(Error::Custom(format!("captcha not found")))?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }

        if let Some(time) = session.get::<i64>("time") {
            let now = Local::now().timestamp();
            if now - time < 60 {
                return Err(Error::Frequent);
            }
        }

        session
    };

    let mut rng = thread_rng();

    let mut code = String::from("");
    for _ in 0..6 {
        code.push_str(&format!("{}", rng.gen_range(0..9)));
    }

    session.expire_in(Duration::from_secs(60 * 5));

    session
        .insert("code", code.clone())
        .map_err(|e| Error::Custom(format!("new session error: {}", e)))?;

    session
        .insert("time", Local::now().timestamp())
        .map_err(|e| Error::Custom(format!("new session error: {}", e)))?;

    tokio::spawn(async move {
        let subject = "验证码/Verify code".to_string();
        let body = format!(
            "<h5>email验证码: {}<h5><br><h5>email verify code: {}<h5>",
            &code, &code,
        );
        util::send_email(payload.email, subject, body)
    });

    let rsp = SendEmailCodeRsp {
        error: E_SUCCESS,
        message: "success".to_string(),
    };

    Ok(Json(rsp))
}
