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
use chrono::Local;
use jsonwebtoken::{decode, Validation};

use crate::{
    app_state::AppState,
    config::CONFIG,
    errors::Error,
    jwt::{Claims, JWT_KEY},
    model::{product::Product, product_user::ProductUser},
};

#[derive(Debug)]
pub struct AuthUser {
    pub product: Product,
    pub user: ProductUser,
    pub new_token: Option<String>,
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
        // Decode the user data
        let claims = decode::<Claims>(bearer.token(), &JWT_KEY.decoding, &Validation::default())
            .map_err(|_| Error::TokenInvalid)?
            .claims;

        let now = Local::now().timestamp();
        if claims.exp < now {
            return Err(Error::TokenInvalid);
        }

        let state = AppState::from_ref(state);

        let product = {
            let products = state.repo.query_products(claims.user_id).await?;
            let products: Vec<_> = products
                .into_iter()
                .filter(|product| product.id.unwrap() == claims.product_id)
                .collect();
            if products.is_empty() {
                return Err(Error::TokenInvalid);
            }
            (*products.get(0).unwrap()).clone()
        };

        let user = state
            .repo
            .query_product_user(claims.product_id, claims.user_id)
            .await?;

        let new_token = if claims.exp - now < CONFIG.jwt_refresh {
            Some(Claims::token(user.product_id, user.user_id)?)
        } else {
            None
        };

        Ok(AuthUser {
            product,
            user,
            new_token,
        })
    }
}
