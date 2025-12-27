use defs::Similarity;
use defs::Similarity::{Cosine, Euclidean, Hamming, Manhattan};
pub const ENV_HOST: &str = "GRPC_SERVER_HOST";
pub const ENV_PORT: &str = "GRPC_SERVER_PORT";
pub const ENV_ROOT_PASSWORD: &str = "GRPC_SERVER_ROOT_PASSWORD";
pub const ENV_STORAGE_TYPE: &str = "GRPC_SERVER_STORAGE_TYPE";
pub const ENV_INDEX_TYPE: &str = "GRPC_SERVER_INDEX_TYPE";
pub const ENV_DIMENSION: &str = "GRPC_SERVER_DIMENSION";
pub const ENV_DATA_PATH: &str = "GRPC_SERVER_DATA_PATH";
pub const ENV_LOGGING: &str = "GRPC_SERVER_LOGGING";

pub const DEFAULT_PORT: &str = "8080";

pub const SIMILARITY_PROTOBUFF_MAP: [Similarity; 4] = [Euclidean, Manhattan, Hamming, Cosine];
pub const AUTHORIZATION_HEADER_KEY: &str = "authorization";
