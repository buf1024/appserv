use axum::{debug_handler, extract::State, Json};
use axum_extra::{extract::WithRejection, headers, TypedHeader};
use regex::Regex;

use crate::{
    app_state::AppState,
    errors::{Error, E_SUCCESS},
    handler::{ok_with_trace, COOKIE_NAME},
    proto::{SignUpReq, SignUpRsp},
    util, JsonRejection, JsonResult,
};
use async_session::SessionStore;

#[debug_handler(state = AppState)]
pub async fn signup(
    State(state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    WithRejection(Json(payload), _): JsonRejection<SignUpReq>,
) -> JsonResult<SignUpRsp> {
    {
        tracing::info!("\nreq: {:?}\n", &payload);

        if payload.email.is_empty()
            || payload.passwd.is_empty()
            || payload.captcha.is_empty()
            || payload.product.is_empty()
        {
            return Err(Error::Parse(String::from(
                "some field is empty, please check.",
            )));
        }
        let re: Regex = Regex::new(r"(?x)^(?P<name>[^@\s]+)@([[:word:]]+\.)*[[:word:]]+$").unwrap();
        re.captures(&payload.email)
            .and_then(|cap| cap.name("name").map(|name| name.as_str()))
            .ok_or(Error::ParseEmail)?;

        if payload.passwd.len() < 6 {
            return Err(Error::UserPasswordTooShort);
        }

        let cookie = cookies.get(COOKIE_NAME).ok_or(Error::Captcha)?;

        let session = state
            .store
            .load_session(cookie.to_string())
            .await
            .map_err(|_| Error::Captcha)?
            .ok_or(Error::Captcha)?;

        let captcha: String = session.get("captcha").ok_or(Error::Captcha)?;

        let code: String = session.get("code").ok_or(Error::Captcha)?;

        let email: String = session.get("email").ok_or(Error::Captcha)?;

        if captcha.to_lowercase() != payload.captcha.to_lowercase() {
            return Err(Error::Captcha);
        }

        if code.to_lowercase() != payload.code.to_lowercase() {
            return Err(Error::EmailVerifyCode);
        }

        if email != payload.email {
            return Err(Error::EmailDiff);
        }

        state
            .store
            .destroy_session(session)
            .await
            .map_err(|e| Error::Internal(format!("destroy session error: {}", e)))?;
    }

    let user = state.repo.create_user(&payload).await?;
    tracing::info!("new create user: {:?}", &user);
    {
        let user_products = state.repo.query_user_products(user.id.unwrap()).await?;

        let mut products = String::new();
        for product in user_products {
            let tmp = format!("<li>{}</li>", product.product);
            products.push_str(tmp.as_str());
        }
        products = format!("<ul>{}</ul>", products);
        tokio::spawn(async move {
            let subject = "注册成功/Signup Success".to_string();
            let body = format!(
                "<h5>账户注册成功，感谢使用!<h5><div>当前已经注册产品: </div><div>{}</div><br><h5>Signup Success, Thank you for interest!<h5>",
            products);
            util::send_email(user.email, subject, body)
        });
    }
    let rsp = SignUpRsp {
        error: E_SUCCESS,
        message: "success".to_string(),
    };

    ok_with_trace(rsp)
}
