use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub listen_addr: SocketAddr,
    pub db_path: PathBuf,
    pub vector_dimension: usize,
}

impl Config {
    pub fn from_env() -> Self {
        // Load listen address
        let listen_addr_str =
            env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let listen_addr = listen_addr_str
            .parse()
            .expect("Failed to parse LISTEN_ADDR");

        // Load database path
        let db_path_str = env::var("DB_PATH").unwrap_or_else(|_| "./data/vectordb".to_string());
        let db_path = PathBuf::from(db_path_str);

        // Load vector dimension
        let vector_dimension_str = env::var("VECTOR_DIMENSION").unwrap_or_else(|_| "3".to_string());
        let vector_dimension = vector_dimension_str
            .parse()
            .expect("Failed to parse VECTOR_DIMENSION");

        Self {
            listen_addr,
            db_path,
            vector_dimension,
        }
    }
}
