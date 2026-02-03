use actix_web::{web, HttpResponse, Responder};
use crate::config::Config;
use crate::services::FileService;

pub async fn list_files(
    path: web::Query<std::collections::HashMap<String, String>>,
    context: web::Path<String>,
    config: web::Data<Config>,
) -> impl Responder {
    let context_name = context.into_inner();
    let context_config = match config.get_context(&context_name) {
        Some(c) => c,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "code": "CONTEXT_NOT_FOUND",
                "message": format!("Context '{}' not found", context_name)
            }));
        }
    };

    let relative_path = path.get("path").map(|p| p.as_str()).unwrap_or("");
    let service = FileService::new(context_config.path.clone());

    match service.list_files(relative_path).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => {
            let status = if e.kind() == std::io::ErrorKind::NotFound {
                actix_web::http::StatusCode::NOT_FOUND
            } else {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            HttpResponse::build(status).json(serde_json::json!({
                "code": "FILE_ERROR",
                "message": e.to_string()
            }))
        }
    }
}

pub async fn get_tree(
    context: web::Path<String>,
    config: web::Data<Config>,
) -> impl Responder {
    let context_name = context.into_inner();
    let context_config = match config.get_context(&context_name) {
        Some(c) => c,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "code": "CONTEXT_NOT_FOUND",
                "message": format!("Context '{}' not found", context_name)
            }));
        }
    };

    let service = FileService::new(context_config.path.clone());

    match service.tree().await {
        Ok(tree) => HttpResponse::Ok().json(tree),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "code": "FILE_ERROR",
            "message": e.to_string()
        })),
    }
}

pub async fn get_content(
    path: web::Query<std::collections::HashMap<String, String>>,
    context: web::Path<String>,
    config: web::Data<Config>,
) -> impl Responder {
    let context_name = context.into_inner();
    let context_config = match config.get_context(&context_name) {
        Some(c) => c,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "code": "CONTEXT_NOT_FOUND",
                "message": format!("Context '{}' not found", context_name)
            }));
        }
    };

    let relative_path = match path.get("path") {
        Some(p) => p.as_str(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "code": "MISSING_PATH",
                "message": "Query parameter 'path' is required"
            }));
        }
    };

    let service = FileService::new(context_config.path.clone());

    match service.get_content(relative_path).await {
        Ok(content) => HttpResponse::Ok().content_type("text/markdown").body(content),
        Err(e) => {
            let status = match e.kind() {
                std::io::ErrorKind::NotFound => actix_web::http::StatusCode::NOT_FOUND,
                std::io::ErrorKind::PermissionDenied => actix_web::http::StatusCode::FORBIDDEN,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(serde_json::json!({
                "code": "FILE_ERROR",
                "message": e.to_string()
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/files", web::get().to(list_files))
        .route("/tree", web::get().to(get_tree))
        .route("/content", web::get().to(get_content));
}
