use std::collections::HashMap;
use crate::config::Route;

#[derive(Debug)]
pub struct MatchResult<'a> {
    pub route: &'a Route,
    pub params: HashMap<String, String>,
}

pub fn match_route<'a>(path: &str, routes: &'a [Route]) -> Option<MatchResult<'a>> {
    let path_segments: Vec<&str> = path
        .split('/')
        .filter(|elem| !elem.is_empty())
        .collect();

    routes.iter().find_map(|route| {
        let route_segments: Vec<&str> = route
            .match_path
            .split('/').filter(|s| !s.is_empty()).collect();

        if route_segments.len() != path_segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (r_seg, p_seg) in route_segments.iter().zip(path_segments.iter()) {
            if r_seg.starts_with('{') && r_seg.ends_with('}') {
                let key = &r_seg[1..r_seg.len() - 1];
                params.insert(key.to_string(), p_seg.to_string());
            } else if r_seg != p_seg {
                return None;
            }
        }
        Some(MatchResult { route, params })
    })
}

