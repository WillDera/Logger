use std::io;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError};

#[derive(Serialize, Debug, Clone)]
pub struct Log {
    pub location_id: String,
    pub device_id: String,
    pub log_level: u32,
    pub log_type: String,
    pub log_message: String,
}

#[derive(Serialize, Validate, Deserialize, Debug)]
pub struct LogEventRequest {
    #[validate(length(min = 1, message = "location_id cannot be empty"))]
    pub location_id: String,

    #[validate(length(min = 1, message = "location_id cannot be empty"))]
    pub device_id: String,

    #[validate(range(min = 0, max = 5))]
    pub log_level: u32,

    #[validate(custom = "validate_log_type")]
    pub log_type: String,

    #[validate(length(min = 10, message = "log message too short"))]
    pub log_message: String,
}

#[derive(Serialize)]
pub struct LogEventResponse {
    pub status: u32,
    pub message: String,
}

#[derive(Serialize)]
pub struct Events {
    pub status: u32,
    pub data: Vec<Log>,
    pub limit: u32,
    pub offset: u32,
    pub size: usize,
}

#[derive(Serialize)]
pub struct AppError {
    pub status: String,
    pub error: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct UploadEventRequest {
    #[validate(length(min = 7, message = "BrandID too short"))]
    pub brand_id: String,

    #[validate(length(min = 7, message = "LocationID too short"))]
    pub location_id: String,

    #[validate(length(min = 1, message = "OrderID too short"))]
    pub order_id: Option<String>,
}

fn validate_log_type(value: &str) -> Result<(), ValidationError> {
    if !["warn", "error", "info", "debug"].contains(&value) {
        return Err(ValidationError::new(
            "Invalid log type. Accepted types are ['warn', 'error', 'info', 'debug']",
        ));
    }

    Ok(())
}

// Impls
impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self {
            status: "200".to_string(),
            error: "Something went wrong!".to_string(),
        }
    }
}

impl Log {
    pub fn from_str(json_string: &str) -> Result<Self, std::io::Error> {
        let value: Value = serde_json::from_str(json_string).unwrap();

        Ok(Log {
            location_id: value["location_id"].as_str().unwrap().to_string(),
            device_id: value["device_id"].as_str().unwrap().to_string(),
            log_level: value["log_level"].as_u64().unwrap() as u32,
            log_type: value["log_type"].as_str().unwrap().to_string(),
            log_message: value["log_message"].as_str().unwrap().to_string(),
        })
    }
}
