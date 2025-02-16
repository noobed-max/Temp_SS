//service.rs
use actix_web::{ web, HttpResponse,Error, HttpRequest};
use futures::StreamExt;
use bytes::BytesMut;
use log::{info, error, warn};
use actix_web::error::{ErrorInternalServerError,ErrorBadRequest};
use log_mdc;


use crate::storage::{write_files_to_storage, get_files_from_storage, delete_and_log};
use crate::database::Database;
use crate::util::serializer::{serialize_offset_size, deserialize_offset_size};


fn header_handler(req: HttpRequest) ->  Result<String, Error> {
    let user = req.headers()
        .get("User")
        .ok_or_else(|| ErrorBadRequest("Missing User header"))?
        .to_str()
        .map_err(|_| ErrorBadRequest("Invalid User header value"))?
        .to_string();
    
    log_mdc::insert("user", &user);    
    Ok(user)
}

pub async fn put_service(
    key: String,
    mut payload: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = header_handler(req)?;

    let db = Database::new(&user)?;
    if db.check_key(&key).map_err(ErrorInternalServerError)? {
        warn!("{}, Key already exists", key);
        return Ok(HttpResponse::BadRequest().body("Key already exists"));
    }

    info!("{}, Starting chunk load", key);
    let mut bytes = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(ErrorInternalServerError)?;
        bytes.extend_from_slice(&chunk);
    }

    if bytes.is_empty() {
        error!("No data uploaded with key: {}", key);
        return Ok(HttpResponse::BadRequest().body("No data was uploaded"));
    }

    info!("{}, Total received data size: {} bytes", key, bytes.len());
    info!("{}, Writing file to storage", key);
    let offset_size_list = write_files_to_storage(&user, &bytes)?;
    info!("{}, done Writing file to storage", key);

    if offset_size_list.is_empty() {
        error!("No data in data list with key: {}", key);
        return Ok(HttpResponse::BadRequest().body("No data in data list"));
    }

    info!("{}, Serializing offset and size ", key);

    let offset_size_bytes = serialize_offset_size(&offset_size_list)?;
    info!("{}, done Serializing offset and size", key);
    info!("{}, uploading to database", key);

    db.upload_sql(&key, &offset_size_bytes)
        .map_err(ErrorInternalServerError)?;

    info!("{}, done uploading to database", key);
    Ok(HttpResponse::Ok().body(format!("Data uploaded successfully: key = {}", key)))
}
