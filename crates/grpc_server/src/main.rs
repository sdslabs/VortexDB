use grpc_server::config::GRPCServerConfig;
use grpc_server::service::{VectorDBService, run_server};
use grpc_server::utils::ServerEndpoint;
use std::panic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // load config from environment from environment variables
    let config = GRPCServerConfig::load_config()
        .inspect_err(|err| panic!("Failed to load config: {}", err))
        .unwrap();

    let vector_db_api = api::init_api(config.db_config)
        .inspect_err(|err| panic!("Failed to Init API: {:?}", err))
        .unwrap();

    let vector_db_service = VectorDBService {
        vector_db: vector_db_api,
        logging: config.logging,
    };
    run_server(
        vector_db_service,
        ServerEndpoint::Address(config.addr),
        config.root_password,
    )
    .await?;
    Ok(())
}
