use std::sync::Arc;
use reqwest::Client;
use crate::cache::TtlCache;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
    pub cache: Arc<TtlCache>,
}