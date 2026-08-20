use tonic::{Status, service::Interceptor};
use tracing::{Level, event};

use crate::constants::AUTHORIZATION_HEADER_KEY;

#[derive(Clone)]
pub struct AuthInterceptor {
    root_password: String,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        let auth_token = match req.metadata().get(AUTHORIZATION_HEADER_KEY) {
            Some(t) => t,
            None => return Err(Status::unauthenticated("Invalid credentials")),
        };
        if auth_token
            .to_str()
            .unwrap_or_default()
            .strip_prefix("Bearer ")
            .unwrap_or_default()
            == self.root_password
        {
            Ok(req)
        } else {
            event!(Level::WARN, "Unauthorized Request");
            Err(Status::unauthenticated("Invalid credentials"))
        }
    }
}

impl AuthInterceptor {
    pub fn new(root_password: String) -> AuthInterceptor {
        AuthInterceptor { root_password }
    }
}
