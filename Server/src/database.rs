
// database.rs
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::{warn, info};
use rocksdb::{Options, DB};
use once_cell::sync::OnceCell;

static ROCKSDB_INSTANCE: OnceCell<Arc<DB>> = OnceCell::new();

fn get_db_path() -> PathBuf {
    env::var("DB_FILE").map(PathBuf::from).unwrap_or_else(|_| {
        let default_path = Path::new("metadata").join("rocksdb");
        std::fs::create_dir_all(&default_path).expect("Failed to create metadata directory");
        default_path
    })
}

fn initialize_db() -> Arc<DB> {
    let db_path = get_db_path();
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
    opts.set_optimize_filters_for_hits(true);
    
    Arc::new(DB::open(&opts, db_path).expect("Failed to open RocksDB"))
}

pub struct Database {
    user: String,
}

impl Database {
    pub fn new(user: &str) -> Result<Self, actix_web::Error> {
        Ok(Database {
            user: user.to_string(),
        })
    }

    fn get_db(&self) -> &DB {
        ROCKSDB_INSTANCE.get_or_init(|| initialize_db()).as_ref()
    }

    pub fn check_key(&self, key: &str) -> Result<bool, actix_web::Error> {
        let composite_key = format!("{}|{}", self.user, key);
        self.get_db()
            .get(composite_key.as_bytes())
            .map(|v| v.is_some())
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))
    }

    pub fn check_key_nonexistance(&self, key: &str) -> Result<(), actix_web::Error> {
        if !self.check_key(key)? {
            warn!("Key does not exist: {}", key);
            return Err(actix_web::error::ErrorNotFound(format!(
                "No data found for key: {}, The key does not exist",
                key
            )));
        }
        Ok(())
    }

    pub fn upload_sql(&self, key: &str, offset_size_bytes: &[u8]) -> Result<(), actix_web::Error> {
        let composite_key = format!("{}|{}", self.user, key);
        self.get_db()
            .put(composite_key.as_bytes(), offset_size_bytes)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))
    }
}
