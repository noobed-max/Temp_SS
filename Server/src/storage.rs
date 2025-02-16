
// storage.rs

use std::fs::{OpenOptions, File};
use std::io::{self, Read, Write, Seek, SeekFrom};
use actix_web::Error;
use log::{warn,error, info};
use flatbuffers::{root, FlatBufferBuilder};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::env;


use crate::util::Flatbuffer_Store_generated::store::{FileDataList, FileData, FileDataListArgs, FileDataArgs};


fn get_storage_directory() -> PathBuf {
    // Try to get the storage directory from environment variable
    match env::var("STORAGE_DIRECTORY") {
        Ok(dir) => {
            info!("Using storage directory from environment: {}", dir);
            PathBuf::from(dir)
        }
        Err(_) => {
            warn!("Storage directory not defined in environment");
            // Use default directory "./storage"            
            let default_path = PathBuf::from("storage");
            if !default_path.exists() {
                std::fs::create_dir_all(&default_path)
                    .expect("Failed to create default storage directory");
            }
            info!("Using default storage directory: {}", default_path.display());
            default_path
        }
    }
}

/* OpenFile provides operations for interacting with binary (.bin) files:
 * - Creating new files
 * - Reading existing files
 * - Writing data to files
 * - Managing file seek operations
 */
 struct OpenFile {
    file: File,
}

impl OpenFile {
    fn new(user: &str) -> io::Result<Self> {
        let storage_dir = get_storage_directory();
        let file_path = storage_dir.join(format!("{}.bin", user));
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&file_path)?;
        Ok(Self { file })
    }
    fn write(&mut self, data: &[u8]) -> io::Result<(u64, u64)> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(data)?;
        Ok((offset, data.len() as u64))
    }
    fn read(&mut self, offset: u64, size: u64) -> io::Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0u8; size as usize];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}


pub fn write_files_to_storage(
    user: &str,
    body: &[u8],
) -> Result<Vec<(u64, u64)>, Error> {
    let mut haystack = OpenFile::new(user)?;

    match haystack.write(body) {
        Ok((offset, size)) => {
            info!("Written data at offset {} with size {}", offset, size);
            Ok(vec![(offset, size)])
        }
        Err(e) => {
            error!("Failed to write data to haystack: {}", e);
            Err(ErrorInternalServerError(e))
        }
    }
}
