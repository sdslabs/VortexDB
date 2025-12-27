#[derive(Debug)]
pub enum ConfigError {
    MissingRequiredEnvVar(String),
    InvalidDimension,
    InvalidDataPath,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
