use std::sync::Arc;
use reqwest::Client;
use crate::cache::TtlCache;
use crate::config::RouteConfig;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub route_config: RouteConfig,
    pub cache: Arc<TtlCache>,
}