mod atlas;
mod model;
mod server;
pub mod codegen;
mod font;

use crate::server::Server;
use actix_cors::Cors;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::web::PayloadConfig;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, http, post, web};
use balchug_common::api::{AddImageResponse, OpenProjectResponse, StartProjectResponse, UpdateGroupsRq, UpdateProjectPropertiesRq};
use balchug_common::scenario::Scenario;
use log::{error, info};

pub type CommonError = Box<dyn std::error::Error + Send + Sync>;

fn internal_err(endpoint: &str, err: CommonError) -> actix_web::Error {
    error!("Error on {endpoint}: {err:?}");
    ErrorInternalServerError("Internal Server Error")
}

#[get("/")]
async fn root() -> impl Responder {
    "Balchug Project Backend"
}

#[post("/start")]
async fn start(
    server: web::Data<Server>,
) -> Result<web::Json<StartProjectResponse>, actix_web::Error> {
    let project = server.create_project().map_err(|err| internal_err("start", err))?;
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
    let content = tokio::fs::read(path).await.map_err(|err| internal_err("assets", err.into()))?;
    Ok(content)
}

#[post("/{id}/props")]
async fn update_project_props(path: web::Path<String>, server: web::Data<Server>, rq: web::Json<UpdateProjectPropertiesRq>)
    -> Result<String, actix_web::Error> {
    let id = path.into_inner();
    let guard = server
        .get_project(&id).await
        .ok_or(ErrorNotFound("Project not found"))?;
    info!("Project {} properties update", id);
    server.update_project_props(guard.project, rq.properties.clone())
        .map_err(|err| internal_err("props", err))?;
    Ok(String::from("OK"))
}

#[post("/{id}/image")]
async fn upload_image(
    path: web::Path<String>,
    server: web::Data<Server>,
    body: web::Bytes,
    content_type: web::Header<http::header::ContentType>,
) -> Result<web::Json<AddImageResponse>, actix_web::Error> {
    let id = path.into_inner();
    let guard = server
        .get_project(&id).await
        .ok_or(ErrorNotFound("Project not found"))?;
    info!("Project {} upload {} bytes with type {}", id, body.len(), content_type.0.0);
    let img_type = content_type.0.0.subtype().as_str().to_string();
    let server_clone = server.clone();
    let task = tokio::task::spawn_blocking(move || server_clone.add_image(guard.project, body.as_ref(), &img_type));
    let (thumbs, atlas) = task.await.map_err(|err| internal_err("image", err.into()))?
        .map_err(|err| internal_err("image", err))?;
    Ok(web::Json(AddImageResponse { thumbs, atlas }))
}

#[post("/{id}/groups")]
async fn update_groups(path: web::Path<String>, server: web::Data<Server>, rq: web::Json<UpdateGroupsRq>)
    -> Result<String, actix_web::Error> {
    let id = path.into_inner();
    let guard = server
        .get_project(&id).await
        .ok_or(ErrorNotFound("Project not found"))?;
    info!("Project {} groups update", id);
    server.update_groups_props(guard.project, rq.groups.clone())
        .map_err(|err| internal_err("groups", err))?;
    Ok(String::from("OK"))
}

#[get("/{id}/project")]
async fn get_project(path: web::Path<String>, server: web::Data<Server>)
    -> Result<web::Json<OpenProjectResponse>, actix_web::Error> {
    let id = path.into_inner();
    let guard = server
        .get_project(&id).await
        .ok_or(ErrorNotFound("Project not found"))?;
    let project = guard.project;
    let resp = OpenProjectResponse {
        project_properties: project.props,
        images_thumbs: project.thumbs,
        atlas: project.images_atlas,
        groups: project.groups,
    };
    Ok(web::Json(resp))
}

#[post("/{id}/export")]
async fn export_project(
    path: web::Path<String>,
    server: web::Data<Server>,
    rq: web::Json<Scenario>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();
    let project = server
        .get_project(&id).await
        .ok_or(ErrorNotFound("Project not found"))?;
    let project_id = project.project.id.clone();
    let server_clone = server.clone();
    let task = tokio::task::spawn_blocking(move || server_clone.compile(project.project, rq.0));
    let task_result = task.await.map_err(|err| internal_err("export", err.into()))?;
    Server::compile_clean(&project_id).map_err(|err| internal_err("export", err))?;
    let result = task_result.map_err(|err| internal_err("export", err))?;
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(format!("dist-{}.zip", id))],
    };
    Ok(
        HttpResponse::Ok()
        .insert_header(content_disposition)
        .body(result)
    )
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = Server::default();
    if let Err(err) = server.load() {
        error!("Server load error: {err}");
    }
    let port = 3000;

    info!("Start server at port {port}");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(PayloadConfig::new(5 * 1024 * 1024))
            .app_data(web::Data::new(server.clone()))
            .service(root)
            .service(start)
            .service(assets)
            .service(update_project_props)
            .service(upload_image)
            .service(update_groups)
            .service(get_project)
            .service(export_project)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
