
//serializer.rs
use bincode;
use actix_web::error::ErrorInternalServerError;
use actix_web::Error;


pub fn serialize_offset_size(offset_size_list: &Vec<(u64, u64)>) -> Result<Vec<u8>, actix_web::Error> {
    bincode::serialize(&offset_size_list)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to serialize size list: {}", e)))
    
}

pub fn deserialize_offset_size(bytes: &[u8]) -> Result<Vec<(u64, u64)>, Error> {
    bincode::deserialize(bytes)
        .map_err(|e| ErrorInternalServerError(format!("Failed to deserialize offset list: {}", e)))
}

use actix_web::{guard::Put, App, HttpServer};
use log::info;

/* storage.rs contains functionality for:
 * - Writing files to storage
 * - Managing file offsets and sizes
 * - Retrieving files from storage
 * - Deleting files from storage
 */ 
mod storage;

// Handles management of keys and offset/size lists
mod database;

// Handles serialization and deserialization of file offsets and sizes into binary format for SQL blob storage
mod util;

mod api;
use crate::api::{put};

mod service;
use log4rs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging, server setup, etc.
    log4rs::init_file("server_log.yaml", Default::default()).unwrap();
    info!("Starting server on 127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(put)
    })
    .bind(("0.0.0.0", 9000))?
    .run()
    .await
}
