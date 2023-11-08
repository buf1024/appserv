use std::time::Duration;

use async_session::Session;
use axum::{extract::State, Json};
use captcha::{by_name, CaptchaName, Difficulty};
use rand::{thread_rng, Rng};
use serde::Serialize;

use crate::{
    app_state::AppState,
    errors::{Error, ERR_SUCCESS},
    JsonResult,
};

#[derive(Debug, Serialize)]
pub struct UserProductsResp {
    error: usize,
    captcha_id: String,
    captcha: String,
}

pub async fn user_products(State(_): State<AppState>) -> JsonResult<UserProductsResp> {
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

    let mut session = Session::new();
    session.expire_in(Duration::from_secs(60 * 5));
    session
        .insert(&captcha_id, chars)
        .map_err(|e| Error::Custom(format!("new session error: {}", e)))?;
    // {
    //     state.store
    //         .store_session(session)
    //         .await
    //         .map_err(|e| Error::Custom(format!("store session error: {}", e)))?;
    // }

    // let captcha = captcha
    //     .as_base64()
    //     .ok_or(Error::Custom(String::from("fail to generate captcha")))?;

    let resp = UserProductsResp {
        error: ERR_SUCCESS,
        captcha_id: "".into(),
        captcha: "".into(),
    };

    Ok(Json(resp))
}
