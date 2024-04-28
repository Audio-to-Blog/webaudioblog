mod s3;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{post, Error};
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


#[post("/image", wrap = "HttpAuthentication::bearer(auth::validator)")]
async fn file_save_rest(req: HttpRequest, mut payload: Payload) -> Result<HttpResponse, Error> {
    // let filename = parse_filename_from_uri(&req.uri().to_string())
    //     .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
    let header = req
        .headers()
        .get("Content-Type")
        .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
    let file_fmt = header.to_str().unwrap().replace("image/", "");
    let filename = format!("{}.{}", id::PostId::generate().to_string(), file_fmt);
    let re =
        regex::Regex::new(r"([a-zA-Z0-9\s_\\.\-\(\):])+(.webp|.jpeg|.png|.gif|.jpg|.tiff|.bmp)$")
            .unwrap();
    let valid;
    if re.is_match(&filename) {
        let filepath = format!("./static/images/{}", sanitize_filename::sanitize(&filename));
        let mut f = async_std::fs::File::create(filepath).await?;
        while let Some(chunk) = payload.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).await?;
        }
        valid = true;
    } else {
        valid = false;
    };
    if valid {
        Ok(HttpResponse::Ok().json(json!({
            "url": format!("{}/images/{}", *BASE_URL, filename),
            "deletion_url": format!("{}/delete/{}", *BASE_URL,filename)
        })))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "message": "No valid Image"
        })))
    }
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut f_n = "".to_string();
    let mut valid = false;
    let mut filevec = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field
            .content_disposition()
            .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
        println!("{:?}", content_type);
        let filename = content_type
            .get_filename()
            .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
        let re = regex::Regex::new(
            r"([a-zA-Z0-9\s_\\.\-\(\):])+(.webp|.jpeg|.png|.gif|.jpg|.tiff|.bmp)$",
        )
            .unwrap();
        if re.is_match(filename) {
            let out = filename.split(".").collect::<Vec<&str>>()[1];
            let filename = format!("{}.{}", id::PostId::generate().to_string(), out);
            println!("{}", filename);
            let filepath = format!("./static/images/{}", sanitize_filename::sanitize(&filename));
            f_n = filename.to_string();
            let mut f = async_std::fs::File::create(filepath).await?;
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).await?;
            }
            filevec.push(json!({
                "url": format!("{}/images/{}", *BASE_URL, f_n) ,
                "deletion_url": format!("{}/delete/{}", *BASE_URL,f_n)
            }));
            valid = true;
        } else {
            valid = false;
        }
    }
    //let uri = req.uri();
    if valid {
        Ok(HttpResponse::Ok().json(json!({ "images": filevec })))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "message": "No valid Image"
        })))
    }
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