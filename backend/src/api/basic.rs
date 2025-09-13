use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: u64,
}

#[derive(Serialize)]
struct RootResponse {
    message: String,
    version: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    uptime: u64,
    version: String,
    memory_usage: String,
    active_connections: u32,
}

/// Root endpoint that provides basic information about the API
#[get("/")]
pub async fn root() -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    
    HttpResponse::Ok().json(RootResponse {
        message: "Welcome to T-Force API".to_string(),
        version: version.to_string(),
    })
}

/// Health check endpoint for monitoring
#[get("/health")]
pub async fn health_check() -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: version.to_string(),
        timestamp,
    })
}

/// Metrics endpoint for Prometheus monitoring
#[get("/metrics")]
pub async fn metrics() -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Basic metrics in Prometheus format
    let metrics = format!(
        "# HELP tforce_uptime_seconds Total uptime in seconds\n\
         # TYPE tforce_uptime_seconds counter\n\
         tforce_uptime_seconds {}\n\
         # HELP tforce_version_info Version information\n\
         # TYPE tforce_version_info gauge\n\
         tforce_version_info{{version=\"{}\"}} 1\n\
         # HELP tforce_health_status Health status\n\
         # TYPE tforce_health_status gauge\n\
         tforce_health_status 1\n",
        uptime, version
    );
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}