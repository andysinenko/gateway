use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    #[serde(rename = "match")]
    pub match_path: String,
    pub rewrite: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    pub routes: Vec<Route>,
}