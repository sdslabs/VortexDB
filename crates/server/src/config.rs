use api::DbConfig;
use defs::{ApiKeyEntry, ApiKeyStore, Similarity};
use dotenv::dotenv;
use index::{IndexType, hnsw::HnswConfig, kd_tree::KDTreeConfig};
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use storage::StorageType;
use tracing::{Level, event};

use crate::constants::{
    DEFAULT_GRPC_PORT, DEFAULT_HNSW_EF, DEFAULT_HNSW_EF_CONSTRUCTION, DEFAULT_HNSW_M,
    DEFAULT_HNSW_MAX_LAYER, DEFAULT_HTTP_PORT, DEFAULT_KD_TREE_BALANCE_THRESHOLD,
    DEFAULT_KD_TREE_DELETE_REBUILD_RATIO, ENV_DATA_PATH, ENV_DIMENSION, ENV_DISABLE_HTTP,
    ENV_GRPC_HOST, ENV_GRPC_PORT, ENV_HNSW_EF, ENV_HNSW_EF_CONSTRUCTION, ENV_HNSW_M, ENV_HNSW_M0,
    ENV_HNSW_MAX_LAYER, ENV_HTTP_HOST, ENV_HTTP_PORT, ENV_INDEX_TYPE,
    ENV_KD_TREE_BALANCE_THRESHOLD, ENV_KD_TREE_DELETE_REBUILD_RATIO, ENV_KEYS_FILE, ENV_LOGGING,
    ENV_SIMILARITY, ENV_STORAGE_TYPE,
};
pub use crate::error::{ConfigError, Result};

#[derive(Debug)]
pub struct ServerConfig {
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub api_keys: Arc<ApiKeyStore>,
    pub db_config: DbConfig,
    pub logging: bool,
    pub disable_http: bool,
}

impl ServerConfig {
    pub fn load_config() -> Result<Self> {
        dotenv().ok();

        // HTTP server configuration
        let http_host = env::var(ENV_HTTP_HOST)
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "HTTP_HOST not defined, defaulting to '127.0.0.1'"
                );
            })
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let http_port = env::var(ENV_HTTP_PORT)
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
        let grpc_host = env::var(ENV_GRPC_HOST)
            .inspect_err(|_| {
                event!(
                    Level::WARN,
                    "GRPC_HOST not defined, defaulting to '127.0.0.1'"
                );
            })
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let grpc_port = env::var(ENV_GRPC_PORT)
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

        let keys_file_path =
            env::var(ENV_KEYS_FILE).map_err(|_| ConfigError::MissingRequiredEnvVar {
                var: ENV_KEYS_FILE.to_string(),
            })?;
        let api_keys = Arc::new(load_keys_file(&keys_file_path)?);

        // Storage type
        let storage_type_str = env::var(ENV_STORAGE_TYPE)
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
        let index_type_str = env::var(ENV_INDEX_TYPE)
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
        let dimension: usize = env::var(ENV_DIMENSION)
            .map_err(|_| ConfigError::MissingRequiredEnvVar {
                var: ENV_DIMENSION.to_string(),
            })?
            .parse()
            .map_err(|_| ConfigError::InvalidDimension)?;

        // Data path
        let data_path: PathBuf = if let Ok(data_path_str) = env::var(ENV_DATA_PATH) {
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
        let logging = env::var(ENV_LOGGING)
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        // HTTP server disable flag (default to false, set to true to run only gRPC)
        let disable_http = env::var(ENV_DISABLE_HTTP)
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        // Similarity metric
        let similarity: Similarity = match env::var(ENV_SIMILARITY) {
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

        let hnsw_m = load_usize_env(ENV_HNSW_M, DEFAULT_HNSW_M);
        let hnsw_config = HnswConfig {
            max_connections: hnsw_m,
            max_connections_0: load_usize_env(ENV_HNSW_M0, 2 * hnsw_m),
            max_layer: load_usize_env(ENV_HNSW_MAX_LAYER, DEFAULT_HNSW_MAX_LAYER),
            ef_construction: load_usize_env(ENV_HNSW_EF_CONSTRUCTION, DEFAULT_HNSW_EF_CONSTRUCTION),
            ef: load_usize_env(ENV_HNSW_EF, DEFAULT_HNSW_EF),
        };
        let kd_tree_config = KDTreeConfig {
            balance_threshold: load_f32_env(
                ENV_KD_TREE_BALANCE_THRESHOLD,
                DEFAULT_KD_TREE_BALANCE_THRESHOLD,
            ),
            delete_rebuild_ratio: load_f32_env(
                ENV_KD_TREE_DELETE_REBUILD_RATIO,
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
            api_keys,
            db_config,
            logging,
            disable_http,
        })
    }
}

#[derive(serde::Deserialize)]
struct KeysFile {
    keys: Vec<ApiKeyEntry>,
}

fn load_keys_file(path: &str) -> Result<ApiKeyStore> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::KeysFileRead {
        path: path.to_string(),
        source,
    })?;
    let parsed: KeysFile =
        serde_json::from_str(&contents).map_err(|source| ConfigError::KeysFileParse {
            path: path.to_string(),
            source,
        })?;

    if parsed.keys.is_empty() {
        return Err(ConfigError::KeysFileEmpty {
            path: path.to_string(),
        });
    }

    if parsed.keys.iter().any(|entry| entry.key.is_empty()) {
        return Err(ConfigError::EmptyApiKey {
            path: path.to_string(),
        });
    }

    let store = ApiKeyStore::new(parsed.keys);
    if let Some(key) = store.duplicate_key() {
        return Err(ConfigError::DuplicateApiKey {
            path: path.to_string(),
            key: key.to_string(),
        });
    }

    Ok(store)
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
