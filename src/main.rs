use actix_files as fs;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::Logger};
use aws_config::{self, load_defaults, meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::Client as S3Client;
use futures::{StreamExt, TryStreamExt};
use sanitize_filename::sanitize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex; // Correct import

mod client;
mod upload_file;

use self::{client::Client, upload_file::UploadedFile};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    namespace: Text<String>,

    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

const S3_BUCKET: &str = "transcribe-ids721";
const STATE_MACHINE_ARN: &str = "arn:aws:states:us-east-1:718203338152:stateMachine:transcribe";
const API_URL: &str = "https://wrnqr49qhe.execute-api.us-east-1.amazonaws.com/beta/execution";

#[derive(Serialize, Deserialize)]
struct CallbackData {
    text_result: String,
}

#[derive(Serialize, Deserialize)]
struct ProcessResponse {
    message: String,
    process_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct StatusResponse {
    complete: bool,
    result: Option<String>,
}

struct AppState {
    processing_status: Arc<Mutex<HashMap<String, StatusResponse>>>,
}

#[derive(Serialize, Deserialize)]
struct ExternalApiResponse {
    executionArn: String,
}

async fn upload_file(
    s3_client: web::Data<Client>,
    data: web::Json<CallbackData>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    tracing::info!("entering /upload");
    let namespace = form.namespace.into_inner();
    let files = form.files;

    tracing::info!("namespace = {namespace:?}");
    tracing::info!("tmp_files = {files:?}");

    // make key prefix (make sure it ends with a forward slash)
    let s3_key_prefix = format!("uploads/{namespace}/");

    // upload temp files to s3 and then remove them
    let uploaded_files = s3_client.upload_files(files, &s3_key_prefix).await?;

    Ok(HttpResponse::Ok().json(json!({
        "uploadedFiles": uploaded_files,
        "meta": json!({ "namespace": namespace }),
    })))
}

async fn callback(
    path: web::Path<String>,
    data: web::Json<CallbackData>,
    state: web::Data<AppState>,
) -> impl Responder {
    let process_id = path.into_inner();
    let text_result = data.into_inner().text_result;

    let mut ps = state.processing_status.lock().unwrap();
    if let Some(status) = ps.get_mut(&process_id) {
        *status = StatusResponse {
            complete: true,
            result: Some(text_result),
        };
        return HttpResponse::Ok().json(json!({"status": "success", "data": status.result}));
    }

    HttpResponse::NotFound().json(json!({"error": "Process ID not found"}))
}

async fn process_file(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let filename = path.into_inner();

    if filename.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "Filename is missing"}));
    }

    let client = reqwest::Client::new();
    let input_data = json!({ "filename": format!("s3://{}/{}", S3_BUCKET, filename) });

    let res = client
        .post(API_URL)
        .json(&json!({
            "input": input_data,  // Make sure this matches the exact JSON format required by the AWS API
            "stateMachineArn": STATE_MACHINE_ARN,
        }))
        .send()
        .await;

    match res {
        Ok(response) if response.status().is_success() => {
            let api_response: ExternalApiResponse = response.json().await.unwrap();
            let process_id = api_response
                .executionArn
                .split(":")
                .last()
                .unwrap()
                .to_string();

            let mut status_map = data.processing_status.lock().unwrap();
            status_map.insert(
                process_id.clone(),
                StatusResponse {
                    complete: false,
                    result: None,
                },
            );

            HttpResponse::Ok()
                .json(json!({ "message": "Processing started", "processId": process_id }))
        }
        Ok(response) => {
            HttpResponse::InternalServerError().json(json!({"error": response.status().as_u16()}))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

async fn check_status(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let process_id = match query.get("process_id") {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("process_id is required"),
    };

    let ps = state.processing_status.lock().unwrap();
    if let Some(status) = ps.get(process_id) {
        HttpResponse::Ok().json(status)
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn index() -> impl Responder {
    fs::NamedFile::open_async("./templates/index.html")
        .await
        .unwrap()
}

#[actix_web::main] // This attribute macro will set up the async runtime for you
async fn main() -> std::io::Result<()> {
    let processing_status = Arc::new(Mutex::new(HashMap::new()));

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);


    HttpServer::new(move || {
        let processing_status = Arc::clone(&processing_status);

        App::new()
            .app_data(web::Data::new(AppState { processing_status }))
            .app_data(web::Data::new(s3_client.clone()))
            .wrap(Logger::default())
            .service(web::resource("/upload").route(web::post().to(upload_file)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(fs::Files::new("/static", "./static"))
            .service(web::resource("/process/{filename}").to(process_file))
            .service(web::resource("/callback/{process_id}").to(callback))
            .service(web::resource("/status").to(check_status))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?; // The `.await` will asynchronously wait for the server to finish running
    Ok(())
}
