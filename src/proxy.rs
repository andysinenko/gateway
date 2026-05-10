use axum::{
    extract::{State, Path},
    http::{Request, StatusCode},
    body::Body,
    response::Response,
};
use axum::http::Method;
use crate::{AppState, matcher::match_route};

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, StatusCode> {
    let cache = state.cache;

    let full_path = format!("/api/{}", path);
    tracing::debug!("incoming: {}", full_path);

    let matched = match_route(&full_path, &state.route_config.routes)
        .ok_or(StatusCode::NOT_FOUND)?;

    let rewritten_path = apply_rewrite(&matched.route.rewrite, &matched.params);
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let uri = format!("{}{}{}", matched.route.target, rewritten_path, query);

    let cache_key = format!("{}{}", req.method(), uri);
    let method = req.method().clone();

    if method == Method::PUT || method == Method::DELETE {
        let get_key = format!("{}{}", Method::GET, uri);
        cache.invalidate(&get_key);
    }

    if req.method() == Method::GET {
        // fsresh cash—we pay out immediately
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(Response::builder()
                .status(200)
                .body(Body::from(cached))
                .unwrap());
        }

        // stale cache — return the stale data, refresh in the background
        if let Some(stale) = cache.get_stale(&cache_key) {
            if cache.try_lock_inflight(&cache_key) {
                let cache_clone = cache.clone();
                let client_clone = state.client.clone();
                let uri_clone = uri.clone();
                let key_clone = cache_key.clone();

                tokio::spawn(async move {
                    if let Ok(resp) = client_clone.get(&uri_clone).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                cache_clone.set(key_clone.clone(), bytes.to_vec());
                            }
                        }
                    }
                    cache_clone.unlock_inflight(&key_clone);
                });
            }

            return Ok(Response::builder()
                .status(200)
                .body(Body::from(stale))
                .unwrap());
        }
    }

    tracing::debug!("proxying to: {}", uri);

    let mut builder = state.client.request(req.method().clone(), &uri);

    for (name, value) in req.headers() {
        builder = builder.header(name, value);
    }

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let resp = builder
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("downstream error: {} | is_connect: {} | is_timeout: {}",
                e, e.is_connect(), e.is_timeout());
            StatusCode::BAD_GATEWAY
        })?;

    let status = resp.status();

    let mut response_builder = Response::builder().status(status);

    for (name, value) in resp.headers() {
        response_builder = response_builder.header(name, value);
    }

    let bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    if method == Method::GET && status.is_success() {
        tracing::debug!("cache set: {}", cache_key);
        cache.set(cache_key, bytes.to_vec());
    }

    Ok(response_builder.body(Body::from(bytes)).unwrap())
}

pub fn apply_rewrite(rewrite: &str, params: &std::collections::HashMap<String, String>) -> String {
    let mut result = rewrite.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}