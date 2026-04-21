use axum::{
    extract::{State, Path},
    http::{Request, StatusCode},
    body::Body,
    response::Response,
};
use crate::{AppState, matcher::match_route};

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, StatusCode> {

    let full_path = format!("/api/{}", path);

    tracing::info!("incoming: {}", full_path);

    let matched = match_route(&full_path, &state.config.routes)
        .ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!("route: {:?}", matched.route);
    tracing::info!("params: {:?}", matched.params);

    let rewritten_path = apply_rewrite(&matched.route.rewrite, &matched.params);

    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();

    let uri = format!("{}{}{}", matched.route.target, rewritten_path, query);

    tracing::info!("proxying to: {}", uri);

    let mut builder = state.client.request(req.method().clone(), uri);

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
            tracing::error!("downstream error: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    let mut response_builder = Response::builder()
        .status(resp.status());

    for (name, value) in resp.headers() {
        response_builder = response_builder.header(name, value);
    }
    let bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

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