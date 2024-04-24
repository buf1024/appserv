use std::time::Duration;

use appserv::{app_router::app_router, app_state::AppState, config::CONFIG};
use tokio::{net::TcpListener, signal};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new().await;
    if state.is_err() {
        tracing::error!("create app state error: {}", state.err().unwrap());
        std::process::exit(-1);
    }
    let state = state.unwrap();

    let store = state.store.clone();
    tokio::spawn(async move {
        tracing::info!("clear session task");
        loop {
            // 清理过期
            if let Err(e) = store.cleanup().await {
                tracing::error!(?e);
            }
            debug!("store count: {}", store.count().await);

            // 清理文件

            // 清理过期token

            tokio::time::sleep(Duration::from_secs(CONFIG.clear_interval as u64)).await;
        }
    });

    let app = app_router(state);

    debug!("debug..");
    info!("listening {} ...", &CONFIG.listen);
    let listener = TcpListener::bind(&CONFIG.listen).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
