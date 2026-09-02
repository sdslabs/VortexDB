use api::DbConfig;
use defs::Similarity;
use dotenv::dotenv;
use index::{IndexType, hnsw::HnswConfig, kd_tree::KDTreeConfig};
use snafu::prelude::*;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use storage::StorageType;
use tracing::{Level, event};

const DEFAULT_HTTP_PORT: &str = "3000";
const DEFAULT_GRPC_PORT: &str = "50051";
const DEFAULT_HNSW_M: usize = 16;
const DEFAULT_HNSW_MAX_LAYER: usize = 16;
const DEFAULT_HNSW_EF_CONSTRUCTION: usize = 200;
const DEFAULT_HNSW_EF: usize = 100;
const DEFAULT_KD_TREE_BALANCE_THRESHOLD: f32 = 0.7;
const DEFAULT_KD_TREE_DELETE_REBUILD_RATIO: f32 = 0.25;

#[derive(Debug)]
pub struct ServerConfig {
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub grpc_root_password: String,
    pub db_config: DbConfig,
    pub logging: bool,
    pub disable_http: bool,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ConfigError {
    #[snafu(display("Missing required environment variable: {var}"))]
    MissingRequiredEnvVar { var: String },

    #[snafu(display("Invalid dimension value"))]
    InvalidDimension,

    #[snafu(display("Invalid data path: {source}"))]
    InvalidDataPath { source: std::io::Error },

    #[snafu(display("IO error: {source}"))]
    IoError { source: std::io::Error },

    #[snafu(display("Invalid address: {addr}"))]
    InvalidAddress { addr: String },
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

impl ServerConfig {
    pub fn load_config() -> Result<Self> {
        dotenv().ok();

        // HTTP server configuration
        let http_host = env::var("HTTP_HOST")
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "HTTP_HOST not defined, defaulting to '127.0.0.1'"
                );
            })
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let http_port = env::var("HTTP_PORT")
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "HTTP_PORT not defined, defaulting to {}",
                    DEFAULT_HTTP_PORT
                );
            })
            .unwrap_or_else(|_| DEFAULT_HTTP_PORT.to_string());

        let http_addr: SocketAddr =
            format!("{}:{}", http_host, http_port)
                .parse()
                .map_err(|_| ConfigError::InvalidAddress {
                    addr: format!("{}:{}", http_host, http_port),
                })?;

        // gRPC server configuration
        let grpc_host = env::var("GRPC_HOST")
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "GRPC_HOST not defined, defaulting to '127.0.0.1'"
                );
            })
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let grpc_port = env::var("GRPC_PORT")
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "GRPC_PORT not defined, defaulting to {}",
                    DEFAULT_GRPC_PORT
                );
            })
            .unwrap_or_else(|_| DEFAULT_GRPC_PORT.to_string());

        let grpc_addr: SocketAddr =
            format!("{}:{}", grpc_host, grpc_port)
                .parse()
                .map_err(|_| ConfigError::InvalidAddress {
                    addr: format!("{}:{}", grpc_host, grpc_port),
                })?;

        // gRPC root password (required)
        let grpc_root_password =
            env::var("GRPC_ROOT_PASSWORD").map_err(|_| ConfigError::MissingRequiredEnvVar {
                var: "GRPC_ROOT_PASSWORD".to_string(),
            })?;

        // Storage type
        let storage_type_str = env::var("STORAGE_TYPE")
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "STORAGE_TYPE not defined, defaulting to InMemory"
                );
            })
            .unwrap_or_default();

        let storage_type = match storage_type_str.to_lowercase().as_str() {
            "inmemory" => StorageType::InMemory,
            "rocksdb" => StorageType::RocksDb,
            _ => StorageType::InMemory,
        };

        // Index type
        let index_type_str = env::var("INDEX_TYPE")
            .inspect_err(|_| {
                event!(Level::WARN, "INDEX_TYPE not defined, defaulting to flat");
            })
            .unwrap_or_else(|_| "flat".to_string())
            .to_lowercase();

        let index_type = match index_type_str.as_str() {
            "flat" => IndexType::Flat,
            "kdtree" => IndexType::KDTree,
            "hnsw" => IndexType::HNSW,
            _ => IndexType::Flat,
        };

        // Dimension (required)
        let dimension: usize = env::var("DIMENSION")
            .map_err(|_| ConfigError::MissingRequiredEnvVar {
                var: "DIMENSION".to_string(),
            })?
            .parse()
            .map_err(|_| ConfigError::InvalidDimension)?;

        // Data path
        let data_path: PathBuf = if let Ok(data_path_str) = env::var("DATA_PATH") {
            let path = PathBuf::from(data_path_str);
            fs::create_dir_all(&path).map_err(|e| ConfigError::InvalidDataPath { source: e })?;
            path
        } else {
            let tempbuf = env::temp_dir().join("vectordb");
            fs::create_dir_all(&tempbuf).map_err(|e| ConfigError::IoError { source: e })?;
            event!(
                Level::WARN,
                "DATA_PATH not specified, using temporary directory: {:?}",
                tempbuf
            );
            tempbuf
        };

        // Logging
        let logging = env::var("LOGGING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        // HTTP server disable flag (default to false, set to true to run only gRPC)
        let disable_http = env::var("DISABLE_HTTP")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        // Similarity metric
        let similarity: Similarity = match env::var("SIMILARITY") {
            Ok(val) => match val.to_lowercase().as_str() {
                "cosine" => Similarity::Cosine,
                "euclidean" => Similarity::Euclidean,
                "manhattan" => Similarity::Manhattan,
                "hamming" => Similarity::Hamming,
                other => {
                    event!(
                        Level::WARN,
                        "Unknown SIMILARITY '{}', defaulting to cosine",
                        other
                    );
                    Similarity::Cosine
                }
            },
            Err(_) => {
                event!(Level::WARN, "SIMILARITY not defined, defaulting to cosine");
                Similarity::Cosine
            }
        };

        let hnsw_m = load_usize_env("HNSW_M", DEFAULT_HNSW_M);
        let hnsw_config = HnswConfig {
            max_connections: hnsw_m,
            max_connections_0: load_usize_env("HNSW_M0", 2 * hnsw_m),
            max_layer: load_usize_env("HNSW_MAX_LAYER", DEFAULT_HNSW_MAX_LAYER),
            ef_construction: load_usize_env("HNSW_EF_CONSTRUCTION", DEFAULT_HNSW_EF_CONSTRUCTION),
            ef: load_usize_env("HNSW_EF", DEFAULT_HNSW_EF),
        };
        let kd_tree_config = KDTreeConfig {
            balance_threshold: load_f32_env(
                "KD_TREE_BALANCE_THRESHOLD",
                DEFAULT_KD_TREE_BALANCE_THRESHOLD,
            ),
            delete_rebuild_ratio: load_f32_env(
                "KD_TREE_DELETE_REBUILD_RATIO",
                DEFAULT_KD_TREE_DELETE_REBUILD_RATIO,
            ),
        };

        let db_config = DbConfig {
            storage_type,
            index_type,
            data_path,
            dimension,
            similarity,
            hnsw_config,
            kd_tree_config,
        };

        Ok(ServerConfig {
            http_addr,
            grpc_addr,
            grpc_root_password,
            db_config,
            logging,
            disable_http,
        })
    }
}

fn load_usize_env(name: &str, default: usize) -> usize {
    match env::var(name) {
        Ok(value) => value.parse().unwrap_or_else(|_| {
            event!(
                Level::WARN,
                "{}='{}' is invalid, defaulting to {}",
                name,
                value,
                default
            );
            default
        }),
        Err(_) => default,
    }
}

fn load_f32_env(name: &str, default: f32) -> f32 {
    match env::var(name) {
        Ok(value) => value.parse().unwrap_or_else(|_| {
            event!(
                Level::WARN,
                "{}='{}' is invalid, defaulting to {}",
                name,
                value,
                default
            );
            default
        }),
        Err(_) => default,
    }
}
