//api.rs
use actix_web::{web, post, HttpRequest, HttpResponse,Error };
use log::info;

use crate::service::put_service;

#[actix_web::post("/put/{key}")]
async fn put(
    key: web::Path<String>,
    payload: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    info!("key: {}", key);
    put_service(key.into_inner(), payload, req).await
}