mod atlas;
mod model;
mod server;

use crate::server::Server;
use actix_cors::Cors;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::web::PayloadConfig;
use actix_web::{App, HttpServer, Responder, get, http, post, web};
use balchug_common::api::{AddImageResponse, StartProjectResponse};
use log::info;

pub type CommonError = Box<dyn std::error::Error + Send + Sync>;

#[get("/")]
async fn root() -> impl Responder {
    "Balchug Project Backend"
}

#[post("/start")]
async fn start(
    server: web::Data<Server>,
) -> Result<web::Json<StartProjectResponse>, actix_web::Error> {
    let project = server.create_project().map_err(ErrorInternalServerError)?;
    info!("New project {}", project.id);
    Ok(web::Json(StartProjectResponse {
        project_id: project.id,
    }))
}

#[get("/{id}/assets/{path}")]
async fn assets(path: web::Path<(String, String)>) -> Result<Vec<u8>, actix_web::Error> {
    let (id, path) = path.into_inner();
    info!("Project {} get asset '{}'", id, path);
    let path = format!("./store/{}/{}", id, path.replace("_", "/"));
    let content = tokio::fs::read(path).await.map_err(ErrorInternalServerError)?;
    Ok(content)
}

#[post("/{id}/image")]
async fn upload_image(
    path: web::Path<String>,
    server: web::Data<Server>,
    body: web::Bytes,
    content_type: web::Header<http::header::ContentType>,
) -> Result<web::Json<AddImageResponse>, actix_web::Error> {
    let id = path.into_inner();
    let project = server
        .get_project(&id)
        .ok_or(ErrorNotFound("Project not found"))?;
    info!("Project {} upload {} bytes with type {}", id, body.len(), content_type.0.0);
    let img_type = content_type.0.0.subtype().as_str();
    let (thumbs, atlas) = server
        .add_image(project, body.as_ref(), img_type)
        .map_err(ErrorInternalServerError)?;
    Ok(web::Json(AddImageResponse { thumbs, atlas }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = Server::default();
    let port = 3000;

    info!("Start server at port {port}");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(PayloadConfig::new(2 * 1024 * 1024))
            .app_data(web::Data::new(server.clone()))
            .service(root)
            .service(start)
            .service(assets)
            .service(upload_image)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
