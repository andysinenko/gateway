use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub match_path: String,
    pub rewrite: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub routes: Vec<Route>,
}