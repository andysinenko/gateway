use std::time::{Duration, Instant};
use axum::Router;
use axum::routing::any;
use reqwest::{Client, ClientBuilder};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use std::env;

use crate::app_state::AppState;

mod app_state;
mod proxy;
mod config;
mod matcher;
mod cache;

use std::fs;
use std::sync::Arc;
use axum::http::StatusCode;
use crate::cache::TtlCache;
use crate::config::AppConfig;



#[tokio::main]
async fn main() {
    tracer_subscr();
    tracing::info!("*** My Gateway is starting ***");

    //init cache
    let cache = Arc::new(TtlCache::new(60));
    //task for cache clearence
    cache.start_eviction_task();

    let gw_host = env::var("GW_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let gw_port = env::var("GW_PORT").unwrap_or_else(|_| "3000".to_string());

    let config_path = env::var("GW_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let config_str = fs::read_to_string(config_path).unwrap();
    let config: AppConfig = serde_yaml::from_str(&config_str).unwrap();

    let client:Client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))//timeout for idle sockets being kept-alive
        .connect_timeout(Duration::from_secs(5))
        .connection_verbose(true)//just for test purpose, todo удалить!
        .build()
        .unwrap();

    let state = AppState {
        client,
        config,
        cache,
    };

    let app = Router::new()
        .route("/api/{*path}", any(proxy::handler))
        .with_state(state)
        .layer(TimeoutLayer ::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(5)))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", gw_host, gw_port))
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

fn tracer_subscr() {
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug,hyper_util=debug,reqwest=debug")
        //.with_env_filter("trace")
        .with_ansi(true)
        .init();
}
