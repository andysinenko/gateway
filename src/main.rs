use std::time::{Duration};
use axum::Router;
use axum::routing::any;
use reqwest::Client;
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
use crate::config::RouteConfig;



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
    
    let route_config: RouteConfig = get_route_config().unwrap_or_else(|e| {
        tracing::error!("Failed to load route config: {}", e);
        std::process::exit(1);
    });

    let client:Client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))//timeout for idle sockets being kept-alive
        .connect_timeout(Duration::from_secs(5))
        .connection_verbose(true)//just for test purpose, todo удалить!
        .build()
        .unwrap();

    let state = AppState {
        client,
        route_config,
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

fn get_route_config() -> Result<RouteConfig, Box<dyn std::error::Error>> {
    let route_config_path = env::var("GW_ROUTE_CONFIG_PATH").unwrap_or_else(|_| "route_config.yaml".to_string());

    tracing::info!("Route config file: {}", route_config_path);

    let route_config_str = fs::read_to_string(&route_config_path).map_err(|e| format!("Failed to read config file '{}': {}", route_config_path, e))?;
    let route_config: RouteConfig = serde_yaml::from_str(&route_config_str).map_err(|e| format!("Failed to parse config file '{}': {}", route_config_path, e))?;


    Ok(route_config)
}
