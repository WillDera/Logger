use axum::{
    extract::DefaultBodyLimit,
    http::Method,
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};

pub mod v1;

use self::v1::logs::{create_log_event, get_log_events, get_log_file, upload_logs};

pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let v1_routes = Router::new()
        .route("/log_events", get(get_log_events))
        .route("/log_file", get(get_log_file))
        .route("/log_event", post(create_log_event))
        .route("/upload_logs", post(upload_logs))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(5 * 1024 * 1024));

    Router::new().layer(cors).nest("/v1", v1_routes)
}
