use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use axum_extra::{extract::WithRejection, headers::Cookie, TypedHeader};

use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS},
    handler::COOKIE_NAME,
    jwt::Claims,
    proto::{SignInReq, SignInRsp},
    JsonRejection, Result,
};
use async_session::SessionStore;

#[debug_handler(state = AppState)]
pub async fn signin(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    WithRejection(Json(payload), _): JsonRejection<SignInReq>,
) -> Result<impl IntoResponse> {
    {
        if payload.email.is_empty()
            || payload.passwd.is_empty()
            || payload.captcha.is_empty()
            || payload.product.is_empty()
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
            .map_err(|_| Error::Captcha)?
            .ok_or(Error::Custom(format!("session not found")))?;

        let captcha: String = session.get("captcha").ok_or(Error::Captcha)?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }

        state
            .store
            .destroy_session(session)
            .await
            .map_err(|e| Error::Custom(format!("destroy session error: {}", e)))?;
    }

    let (user, product) = state.repo.signin_user(&payload).await?;
    tracing::info!(?user, ?product);

    let token = Claims::token(product.id.unwrap(), user.id.unwrap())?;
    let rsp = SignInRsp {
        error: E_SUCCESS,
        message: "success".to_string(),
        token,
    };

    Ok(Json(rsp))
}
