mod signup;
use std::time::Duration;

use async_session::{MemoryStore, SessionStore};
use async_trait::async_trait;
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    headers,
    http::{header, request::Parts},
    RequestPartsExt, TypedHeader,
};
pub use signup::signup;

mod captcha;
pub use captcha::captcha;

mod user_products;
pub use user_products::user_products;

mod activate;
pub use activate::activate;

mod signin;
pub use signin::signin;

mod is_signin;
pub use is_signin::is_signin;

mod user_info;
pub use user_info::user_info;

mod signout;
pub use signout::signout;

use crate::{errors::Error, model::user::User};

pub const COOKIE_NAME: &'static str = "SESSION";

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => Error::UserNotLogin,
                    _ => panic!("unexpected error getting Cookie header(s): {e}"),
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME);
        if session_cookie.is_none() {
            return Err(Error::UserNotLogin);
        }

        let session = store
            .load_session(session_cookie.unwrap().to_string())
            .await
            .map_err(|_| Error::UserNotLogin)?;

        if session.is_none() {
            return Err(Error::UserNotLogin);
        }
        let mut session = session.unwrap();

        let user = session.get::<User>("user");

        if user.is_none() {
            return Err(Error::UserNotLogin);
        }

        session.expire_in(Duration::from_secs(60 * 5));
        let user = user.unwrap();

        Ok(user)
    }
}
