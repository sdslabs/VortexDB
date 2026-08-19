use std::sync::Arc;

use defs::{ApiKeyEntry, ApiKeyRole, ApiKeyStore};
use tonic::{Status, service::Interceptor};
use tracing::{Level, event};

use crate::constants::AUTHORIZATION_HEADER_KEY;

#[derive(Clone)]
pub struct AuthInterceptor {
    keys: Arc<ApiKeyStore>,
}

fn extract_bearer_token<T>(req: &tonic::Request<T>) -> Option<&str> {
    req.metadata()
        .get(AUTHORIZATION_HEADER_KEY)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

pub fn require_write_role<T>(req: &tonic::Request<T>) -> Result<(), Status> {
    match req.extensions().get::<ApiKeyEntry>() {
        Some(entry) if entry.role == ApiKeyRole::ReadWrite => Ok(()),
        Some(_) => Err(Status::permission_denied(
            "This api key does not have write access",
        )),
        None => Err(Status::unauthenticated("Invalid credentials")),
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let matched = extract_bearer_token(&req)
            .and_then(|token| self.keys.find(token))
            .cloned();

        match matched {
            Some(entry) => {
                req.extensions_mut().insert(entry);
                Ok(req)
            }
            None => {
                event!(Level::WARN, "Unauthorized Request");
                Err(Status::unauthenticated("Invalid credentials"))
            }
        }
    }
}

impl AuthInterceptor {
    pub fn new(keys: Arc<ApiKeyStore>) -> AuthInterceptor {
        AuthInterceptor { keys }
    }
}
