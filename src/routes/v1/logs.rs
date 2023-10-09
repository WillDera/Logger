use std::{
    fs::{self, create_dir_all, File},
    io::{BufWriter, Write},
    path::Path,
};

use crate::dto::{
    logs::{AppError, Events, Log, LogEventRequest, LogEventResponse, UploadEventRequest},
    utils::{extract_data, get_latest_file_name, Pagination},
};
use axum::{
    body::StreamBody,
    extract::{Multipart, Query},
    http::{header, status},
    response::IntoResponse,
    Json,
};
use byte_unit::Byte;
use chrono::prelude::*;
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
// use mime_guess::from_path;
use serde_json::json;
use tokio_util::io::ReaderStream;
use validator::Validate;

pub async fn get_log_events(page: Query<Pagination>) -> Result<Json<Events>, Json<AppError>> {
    // validate request
    if let Some(err) = page.validate().err() {
        return Err(Json(AppError {
            status: status::StatusCode::BAD_REQUEST.to_string(),
            error: err.to_string(),
        }));
    }

    // serialize event_logs
    //TODO: Add error handling here
    let mut data: Vec<Log> = Vec::new();
    let reader: String = fs::read_to_string("log_folder/event_logs")
        .unwrap()
        .parse()
        .unwrap();
    for line in reader.lines() {
        data.push(Log::from_str(line).unwrap());
    }

    let full_data: Vec<_> = data.into_iter().rev().collect();
    let data = extract_data(full_data.as_ref(), page.limit, page.offset);

    Ok(Json(Events {
        status: status::StatusCode::OK.as_u16() as u32,
        data: data.to_vec(),
        limit: page.limit,
        offset: page.offset,
        size: full_data.len(),
    }))
}

pub async fn get_log_file(
    query: Query<UploadEventRequest>,
) -> Result<impl IntoResponse, Json<AppError>> {
    // construct path
    let path = match query.order_id {
        Some(_) => format!(
            "log_folder/{}/{}/{}",
            query.brand_id,
            query.location_id,
            query.order_id.clone().unwrap().to_string()
        ),
        None => format!("log_folder/{}/{}", query.brand_id, query.location_id),
    };

    // find last modified file
    let filename = get_latest_file_name(path.as_str());

    let new_path = Path::new(&path).join(filename.as_ref().unwrap());

    let file = match tokio::fs::File::open(new_path.clone()).await {
        Ok(file) => file,
        Err(_) => {
            return Err(Json(AppError {
                status: status::StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                error: "Tokio: Error Opening File".to_string(),
            }))
        }
    };

    // let content_type = match from_path(new_path).first_raw() {
    //     Some(mime) => mime,
    //     None => {
    //         return Err(Json(AppError {
    //             status: status::StatusCode::INTERNAL_SERVER_ERROR.to_string(),
    //             error: "MIME Type couldn't be determined".to_string(),
    //         }))
    //     }
    // };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let headers = [
        (
            header::CONTENT_TYPE,
            "application/octet-stream; charset=utf-8",
        ),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{:?}\"", filename),
        ),
    ];

    Ok((headers, body).into_response())
}

pub async fn create_log_event(
    Json(body): Json<LogEventRequest>,
) -> Result<Json<LogEventResponse>, Json<AppError>> {
    // validate request
    if let Some(err) = body.validate().err() {
        return Err(Json(AppError {
            status: status::StatusCode::BAD_REQUEST.to_string(),
            error: err.to_string(),
        }));
    }

    // Instantiate File Rotation
    let max_size = Byte::from_str("3 MB").unwrap();
    let mut log = FileRotate::new(
        "log_folder/event_logs",
        AppendCount::new(100),
        ContentLimit::Bytes(max_size.get_bytes() as usize),
        Compression::None,
        None,
    );

    let mut output = vec![];
    output.push(json!(body));

    // write request body to the log file or throw an error
    match writeln!(log, "{}", format!("{}", (json!(body)))) {
        Ok(_) => Ok(Json(LogEventResponse {
            status: 200,
            message: "Data Logged Successfully!".to_string(),
        })),
        Err(e) => Err(Json(AppError {
            status: status::StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            error: format!("Logging Failed! With Error: {:?}", e).to_string(),
        })),
    }
}

pub async fn upload_logs(
    query: Query<UploadEventRequest>,
    mut multipart: Multipart,
) -> Result<Json<LogEventResponse>, Json<AppError>> {
    // validate request
    if let Some(err) = query.validate().err() {
        return Err(Json(AppError {
            status: status::StatusCode::BAD_REQUEST.to_string(),
            error: err.to_string(),
        }));
    }

    let name: String;

    // store file
    if let Some(field) = multipart.next_field().await.unwrap() {
        name = field.name().unwrap().to_string();
        let _file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        let path = match query.order_id {
            Some(_) => format!(
                "log_folder/{}/{}/{}",
                query.brand_id,
                query.location_id,
                query.order_id.clone().unwrap().to_string()
            ),
            None => format!("log_folder/{}/{}", query.brand_id, query.location_id),
        };

        let _ = create_dir_all(path.clone());
        let tmp_dir = Path::new(&path).join(name.clone() + "_" + &Utc::now().to_string());

        let mut file_writer = BufWriter::new(File::create(&tmp_dir).unwrap());
        let _ = file_writer.write_all(&data);
    } else {
        return Err(Json(AppError {
            status: status::StatusCode::OK.to_string(),
            error: "No File Uploaded or Something went wrong".to_string(),
        }));
    }

    // Response
    Ok(Json(LogEventResponse {
        status: 200,
        message: format!("Log: {} Successful", name),
    }))
}
