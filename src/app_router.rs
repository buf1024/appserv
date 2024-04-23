use std::time::{Duration, Instant};

use axum::{
    error_handling::HandleErrorLayer,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    BoxError, Router,
};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Level;

use crate::{
    app_state::AppState,
    handler::{captcha, send_email_code, signin, signup, products},
};

pub fn app_router(state: AppState) -> Router {
    // user
    // hiqradio
    // let router = Router::new()
    //     .route("/signup", post(signup))
    //     .route("/captcha", get(captcha))
    //     .route("/activate", get(activate))
    //     .route("/signin", post(signin))
    //     .route("/is_signin", post(is_signin))
    //     .route("/products", post(signup))
    //     .route("/signout", post(signout))
    //     .route("/user/info", post(user_info))
    //     .route("/user/products", post(user_products).get(user_products))
    //     .route("/user/subscribe", get(captcha));

    // common
    let router_common = Router::new()
        .route("/captcha", get(captcha))
        .route("/verify_email", post(send_email_code));

    let router_common = Router::new().nest("/common", router_common);

    // user
    let router_user = Router::new()
        .route("/signup", post(signup))
        .route("/signin", post(signin))
        .route("/products", post(products));

    let router_user = Router::new().nest("/user", router_user);


    Router::new()
        .nest("/api", router_common)
        .nest("/api", router_user)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(CompressionLayer::new())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO)),
                )
                .layer(middleware::from_fn(track_request)),
        )
        .with_state(state)
}

async fn track_request(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    // let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
    //     matched_path.as_str().to_owned()
    // } else {
    //     req.uri().path().to_owned()
    // };
    // let method = req.method().clone();
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    // let status = response.status().as_u16().to_string();

    tracing::info!(?latency);

    response
}
