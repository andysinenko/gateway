use reqwest::Client;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
}