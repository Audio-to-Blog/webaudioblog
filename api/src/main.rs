use actix_files as fs;

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Responder};



use serde_json::json;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

mod client;

mod upload_file;

use upload_file::UploadedFile;
use client::Client;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    namespace: Text<String>,

    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/upload")]
async fn upload_to_s3(
    s3_client: web::Data<Client>,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    std::fs::create_dir_all("./tmp").unwrap();

    tracing::info!("Starting service at 127.0.0.1:8000");
    HttpServer::new(move || {
        App::new()
            .service(fs::Files::new("/", "../templates").index_file("index.html"))
            // .route("/upload", web::post().to(upload_to_s3))
            // .route("/process/{filename}", web::post().to(process_file))
            .service(upload_to_s3)
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(s3_client.clone()))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
