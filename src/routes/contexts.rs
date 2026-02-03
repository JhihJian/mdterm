use actix_web::{web, HttpResponse, Responder};
use crate::config::Config;

pub async fn list_contexts(config: web::Data<Config>) -> impl Responder {
    let contexts: Vec<_> = config
        .contexts
        .iter()
        .map(|c| serde_json::json!({
            "name": c.name,
            "path": c.path,
            "description": c.description,
        }))
        .collect();

    HttpResponse::Ok().json(contexts)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/contexts", web::get().to(list_contexts));
}
