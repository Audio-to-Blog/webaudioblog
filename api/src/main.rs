mod s3;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::str::FromStr;
use uuid::Uuid;
use reqwest::Client;
use serde_json::json;
use std::io::Read;
use std::io::Write;
use s3::PutFile;
use aws_sdk_s3::Client as S3Client;


async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

struct AppState {
    s3_client: S3Client,
}

async fn upload_file(mut payload: Multipart, data: web::Data<AppState>) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap().parameters[0].clone().value.unwrap();
        let filename = format!("{}-{}", Uuid::new_v4(), field.content_disposition().unwrap().get_filename().unwrap());
        let mut file = web::block(|| std::fs::File::create(&filename)).await.unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file = web::block(move || file.unwrap().write_all(&data).map(|_| file)).await.unwrap();
        }

        // Read the file into a Vec<u8>
        let mut file = web::block(|| std::fs::File::open(&filename)).await.unwrap();
        let mut buffer = Vec::new();
        file.unwrap().read_to_end(&mut buffer).unwrap();

        // Use the put_file function from the s3 module
        let result = data.s3_client.put_file("transcribe-ids721", &filename, buffer).await;

        match result {
            Ok(_) => return HttpResponse::Ok().json(json!({ "message": "File uploaded successfully", "filename": filename })),
            Err(_) => return HttpResponse::BadRequest().body("Failed to upload file"),
        }
    }

    HttpResponse::BadRequest().body("Failed to upload file")
}

async fn process_file(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let process_id = Uuid::new_v4().to_string();
    let data = json!({
        "input": format!("s3://transcribe-ids721/{}", filename),
        "name": format!("Execution-{}", process_id),
        "stateMachineArn": "arn:aws:states:us-east-1:718203338152:stateMachine:transcribe"
    });

    let client = Client::new();
    let response = client.post("https://wrnqr49qhe.execute-api.us-east-1.amazonaws.com/beta/execution")
        .json(&data)
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                HttpResponse::Ok().json(json!({ "message": "Processing started", "processId": process_id }))
            } else {
                HttpResponse::BadRequest().json(json!({ "error": "Error initiating processing" }))
            }
        }
        Err(_) => HttpResponse::BadRequest().body("Filename is missing"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s3_client = S3Client::new(aws_sdk_s3::config::Region::from_str("us-east-1").unwrap());

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                s3_client: s3_client.clone(),
            })
            .service(fs::Files::new("/static", "public"))
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_file))
            .route("/process/{filename}", web::post().to(process_file))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}