use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{
    app_state::AppState,
    errors::Error,
    model::{product::Product, user::User, user_product::UserProduct},
};

#[derive(Debug)]
pub struct AuthUser {
    pub token: String,
    pub product: Product,
    pub user: User,
    pub user_product: UserProduct,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::TokenInvalid)?;

        let state = AppState::from_ref(state);

        let session = state.repo.get_session(bearer.token()).await?;

        let product = state.repo.query_product(session.product_id).await?;
        let user = state.repo.query_user(session.user_id).await?;
        let user_product = state
            .repo
            .query_user_product(session.user_id, session.product_id)
            .await?;

        Ok(AuthUser {
            product,
            user,
            user_product,
            token: session.token,
        })
    }
}
