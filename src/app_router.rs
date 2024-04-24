use std::time::{Duration, Instant};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    BoxError, Router,
};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Level;

use crate::{
    app_state::AppState,
    handler::{captcha, hiqradio, send_email_code, user},
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
        .route("/send_email_code", post(send_email_code));

    let router_common = Router::new().nest("/common", router_common);

    // user
    let router_user = Router::new()
        .route("/signup", post(user::signup))
        .route("/signin", post(user::signin))
        .route("/signout", post(user::signout))
        .route("/user_info", post(user::user_info))
        .route("/upload", post(user::upload))
        .route("/modify", post(user::modify))
        .route("/user_products", post(user::user_products))
        .route("/products", post(user::products));

    let router_user = Router::new().nest("/user", router_user);

    // hiqradio
    let router_hiqradio = Router::new()
        .route("/recently", post(hiqradio::recently))
        .route("/recently_new", post(hiqradio::recently_new))
        .route("/recently_clear", post(hiqradio::recently_clear))
        .route("/groups", post(hiqradio::groups))
        .route("/group_delete", post(hiqradio::group_delete))
        .route("/group_modify", post(hiqradio::group_modify))
        .route("/group_new", post(hiqradio::group_new))
        .route("/favorites", post(hiqradio::favorites))
        .route("/favorite_delete", post(hiqradio::favorite_delete))
        .route("/favorite_modify", post(hiqradio::favorite_modify))
        .route("/favorite_new", post(hiqradio::favorite_new));

    let router_hiqradio = Router::new().nest("/hiqradio", router_hiqradio);

    Router::new()
        .nest("/api", router_common)
        .nest("/api", router_user)
        .nest("/api", router_hiqradio)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(60)))
                .layer(CompressionLayer::new())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO)),
                )
                .layer(middleware::from_fn(track_request)),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */))
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
