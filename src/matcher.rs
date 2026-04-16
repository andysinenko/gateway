use crate::config::Route;

pub fn match_route<'a>(path: &str, routes: &'a [Route]) -> Option<&'a Route> {
    routes.iter().find(|r| r.match_path == path)
}