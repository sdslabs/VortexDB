use crate::constants::{
    self, DEFAULT_PORT, ENV_DATA_PATH, ENV_DIMENSION, ENV_INDEX_TYPE, ENV_LOGGING, ENV_PORT,
    ENV_ROOT_PASSWORD, ENV_STORAGE_TYPE,
};
use crate::errors;
use api;
use dotenv::dotenv;
use index::IndexType;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::{env, fs};
use storage;
use tempfile::tempdir;
use tracing::{Level, event};

pub struct GRPCServerConfig {
    pub addr: SocketAddr,
    pub root_password: String,
    pub db_config: api::DbConfig,
    pub logging: bool,
}

impl GRPCServerConfig {
    pub fn load_config() -> Result<GRPCServerConfig, Box<dyn std::error::Error>> {
        dotenv().ok();

        // fetch server host; default to localhost if not defined
        let host = env::var(constants::ENV_HOST)
            .inspect_err(|_| {
                event!(Level::WARN, "Host not defined, defaulting to 'localhost'");
            })
            .unwrap_or("127.0.0.1".to_string());

        // fetch server port; default to 8080 if not defined
        let port: u32 = env::var(ENV_PORT)
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "Port not defined, defaulting to {}",
                    DEFAULT_PORT
                );
            })
            .unwrap_or(DEFAULT_PORT.to_string())
            .parse()
            .unwrap_or(DEFAULT_PORT.parse::<u32>().unwrap());

        // fetch server root password; return err if not defined
        let root_password = env::var(ENV_ROOT_PASSWORD).map_err(|_| {
            errors::ConfigError::MissingRequiredEnvVar(ENV_ROOT_PASSWORD.to_string())
        })?;

        // fetch server storage type
        let storage_type_str = env::var(ENV_STORAGE_TYPE)
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "Storage Type not defined, defaulting to InMemory"
                )
            })
            .unwrap_or_default();
        let storage_type = match storage_type_str.as_str() {
            "inmemory" => storage::StorageType::InMemory,
            "rocksdb" => storage::StorageType::RocksDb,
            _ => storage::StorageType::InMemory, // default to InMemory if not specified
        };

        // fetch server index type
        let index_type_str = env::var(ENV_INDEX_TYPE)
            .inspect_err(|_| event!(Level::WARN, "Index Type not defined, defaulting to flat"))
            .unwrap_or("flat".to_string())
            .to_lowercase();
        let index_type = match index_type_str.as_str() {
            "flat" => IndexType::Flat,
            "kdtree" => IndexType::KDTree,
            "hnsw" => IndexType::HNSW,
            _ => IndexType::Flat, // default to Flat if not specified
        };

        // fetch dimension size
        let dimension: usize = env::var(ENV_DIMENSION)
            .map_err(|_| errors::ConfigError::MissingRequiredEnvVar(ENV_DIMENSION.to_string()))?
            .parse()
            .map_err(|_| errors::ConfigError::InvalidDimension)?;

        // fetch data path; create tempdir if not specified
        let data_path: PathBuf;
        if let Ok(data_path_str) = env::var(ENV_DATA_PATH) {
            data_path = PathBuf::from(data_path_str);
            fs::create_dir_all(&data_path).map_err(|_| errors::ConfigError::InvalidDataPath)?;
        } else {
            let tempbuf = tempdir().unwrap().path().to_path_buf().join("vectordb");
            fs::create_dir_all(&tempbuf)?;
            event!(
                Level::WARN,
                "Data Path not specified, using temporary directory: {:?}",
                tempbuf.clone()
            );
            data_path = tempbuf;
        }

        // create db config for api
        let db_config = api::DbConfig {
            storage_type,
            index_type,
            data_path,
            dimension,
        };

        // create socket address for grpc server
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

        // check if logging is enabled
        let mut logging: bool = true; // default to logging enabled
        if let Ok(logging_str) = env::var(ENV_LOGGING) {
            logging = logging_str.parse().unwrap_or(true);
        }

        Ok(GRPCServerConfig {
            addr,
            root_password,
            db_config,
            logging,
        })
    }
}
