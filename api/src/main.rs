use std::env;
use actix_files as fs;
use actix_web::middleware::Compat;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Responder, get};
use actix_web_lab::respond::Html;

use tracing::{Subscriber, subscriber::set_global_default};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};


use serde_json::json;
use uuid::Uuid;

mod client;

mod upload_file;

use upload_file::UploadedFile;
use client::Client;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/upload")]
async fn upload_to_s3(
    s3_client: web::Data<Client>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    tracing::info!("entering /upload");
    let files = form.files;

    tracing::info!("tmp_files = {files:?}");

    // make key prefix (make sure it ends with a forward slash)
    let s3_key_prefix = format!("");

    // upload temp files to s3 and then remove them
    let uploaded_files = s3_client.upload_files(files, &s3_key_prefix).await?;

    Ok(HttpResponse::Ok().json(json!({
        "uploadedFiles": uploaded_files,
    })))
}

async fn save_file() -> Result<HttpResponse, Error> {
    // let mut f_n = "".to_string();
    // let mut valid = false;
    // let mut filevec = Vec::new();
    // while let Ok(Some(mut field)) = payload.try_next().await {
    //     let content_type = field
    //         .content_disposition()
    //         .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
    //     println!("{:?}", content_type);
    //     let filename = content_type
    //         .get_filename()
    //         .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
    //     let re = regex::Regex::new(
    //         r"([a-zA-Z0-9\s_\\.\-():])+(.webp|.jpeg|.png|.gif|.jpg|.tiff|.bmp)$",
    //     )
    //         .unwrap();
    //     if re.is_match(filename) {
    //         let out = filename.split(".").collect::<Vec<&str>>()[1];
    //         let filename = format!("{}.{}", id::PostId::generate().to_string(), out);
    //         println!("{}", filename);
    //         let filepath = format!("./static/images/{}", sanitize_filename::sanitize(&filename));
    //         f_n = filename.to_string();
    //         let mut f = async_std::fs::File::create(filepath).await?;
    //         while let Some(chunk) = field.next().await {
    //             let data = chunk.unwrap();
    //             f.write_all(&data).await?;
    //         }
    //         filevec.push(json!({
    //             "url": format!("{}/images/{}", *BASE_URL, f_n) ,
    //             "deletion_url": format!("{}/delete/{}", *BASE_URL,f_n)
    //         }));
    //         valid = true;
    //     } else {
    //         valid = false;
    //     }
    // }
    // //let uri = req.uri();
    // if valid {
    //     Ok(HttpResponse::Ok().json(json!({ "images": filevec })))
    // } else {
    //     Ok(HttpResponse::BadRequest().json(json!({
    //         "message": "No valid Image"
    //     })))
    // }

    todo!()
}

async fn process_file(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let process_id = Uuid::new_v4().to_string();
    let _data = json!({
        "input": format!("s3://transcribe-ids721/{}", filename),
        "name": format!("Execution-{}", process_id),
        "stateMachineArn": "arn:aws:states:us-east-1:718203338152:stateMachine:transcribe"
    });

    // let client = Client::new();
    // let response = client.post("https://wrnqr49qhe.execute-api.us-east-1.amazonaws.com/beta/execution")
    //     .json(&data)
    //     .send()
    //     .await;
    //
    // match response {
    //     Ok(res) => {
    //         if res.status().is_success() {
    //             HttpResponse::Ok().json(json!({ "message": "Processing started", "processId": process_id }))
    //         } else {
    //             HttpResponse::BadRequest().json(json!({ "error": "Error initiating processing" }))
    //         }
    //     }
    //     Err(_) => HttpResponse::BadRequest().body("Filename is missing"),
    // }
    HttpResponse::Ok()
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    name: String,
    env_filter: String
) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(
        name.into(),
        std::io::stdout
    );
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[get("/")]
async fn index() -> impl Responder {
    Html(include_str!("../../templates/index.html").to_owned())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    std::fs::create_dir_all("./tmp").unwrap();

    let subscriber = get_subscriber("app".into(), "debug".into());
    init_subscriber(subscriber);

    tracing::info!("Starting service at 127.0.0.1:8000");
    HttpServer::new(move || {
        App::new()
            .service(index)
            // .route("/upload", web::post().to(upload_to_s3))
            .route("/process/{filename}", web::post().to(process_file))
            .service(upload_to_s3)
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(client.clone()))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
