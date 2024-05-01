use std::{fs, time::Duration};

use appserv::{app_router::app_router, app_state::AppState, config::CONFIG};
use chrono::{Datelike, Days, Local, TimeZone};
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::daily(&CONFIG.log_path, "appserv.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
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
            // 清理过期数据
            if let Err(e) = store.cleanup().await {
                tracing::info!("clean session");
                tracing::error!(?e);
            }
            tracing::debug!("store count: {}", store.count().await);

            // 清理文件

            // 清理过期token

            tokio::time::sleep(Duration::from_secs(CONFIG.session_interval as u64)).await;
        }
    });

    let repo = state.repo.clone();
    tokio::spawn(async move {
        tracing::info!("file and token clear task");

        let start_time_fn = || {
            let now = Local::now();
            let now = now.checked_add_days(Days::new(1)).unwrap();

            let next = chrono::Local
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 15, 0)
                .unwrap();

            tracing::info!(
                "next clean time: {}-{:02}-{:02} 00:15:00",
                next.year(),
                next.month(),
                next.day()
            );
            next.timestamp()
        };

        let mut start_time = 0;
        loop {
            let now = Local::now().timestamp();
            if now >= start_time {
                // 清理文件
                tracing::info!("clean avatar file..");
                if let Ok(read_dir) = fs::read_dir(&CONFIG.avatar_path) {
                    for e in read_dir {
                        if let Ok(entry) = e {
                            if entry.path().is_file() {
                                if let Some(file_name) = entry.file_name().to_str() {
                                    if let Err(e) = repo.clean_avatar_path(file_name).await {
                                        tracing::error!("clean avatar error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }

                // 清理过期token
                tracing::info!("clean expire token..");
                if let Err(e) = repo.clean_session().await {
                    tracing::error!("clean avatar error: {}", e);
                }

                start_time = start_time_fn();
            }

            tokio::time::sleep(Duration::from_secs(CONFIG.clean_interval as u64)).await;
        }
    });

    let app = app_router(state);

    tracing::debug!("debug..");
    tracing::info!("listening {} ...", &CONFIG.listen);
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
