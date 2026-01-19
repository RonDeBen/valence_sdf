use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Request, header},
    middleware::{self, Next},
    response::Response,
    routing,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let static_files =
        ServeDir::new("../dist").not_found_service(ServeFile::new("../dist/index.html"));

    let app = Router::new()
        .route("/healthz", routing::get(healthz))
        .fallback_service(static_files)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new().br(true).gzip(true))
                .layer(middleware::from_fn(cache_control)),
        );

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Server running on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn cache_control(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned(); // <- avoid borrowing req
    let mut res = next.run(req).await;

    // HTML: always revalidate (lets you change index each load)
    if path == "/" || path.ends_with(".html") {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, must-revalidate"),
        );
        return res;
    }

    // Assets:
    //    - If fingerprinted: cache "forever"
    //    - Otherwise: cache, but always revalidate
    let value = if is_fingerprinted_asset(&path) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=0, must-revalidate"
    };

    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    res
}

// Heuristic: treat "foo.<hash>.wasm/js/css" as fingerprinted.
fn is_fingerprinted_asset(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let mut parts = file.split('.');

    // need at least name.hash.ext  => 3 parts minimum
    let first = parts.next();
    let second = parts.next();
    let third = parts.next();

    if first.is_none() || second.is_none() || third.is_none() {
        return false;
    }

    let hash = second.unwrap();
    hash.len() >= 8 && hash.chars().all(|c| c.is_ascii_hexdigit())
}
