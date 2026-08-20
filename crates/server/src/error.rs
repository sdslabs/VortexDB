use snafu::prelude::*;

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

    #[snafu(display("Failed to read keys file {path}: {source}"))]
    KeysFileRead {
        path: String,
        source: std::io::Error,
    },

    #[snafu(display("Failed to parse keys file {path}: {source}"))]
    KeysFileParse {
        path: String,
        source: serde_json::Error,
    },

    #[snafu(display("Keys file {path} contains no keys"))]
    KeysFileEmpty { path: String },

    #[snafu(display("Keys file {path} contains an entry with an empty key value"))]
    EmptyApiKey { path: String },

    #[snafu(display("Keys file {path} has two entries with the same key value: {key}"))]
    DuplicateApiKey { path: String, key: String },
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
