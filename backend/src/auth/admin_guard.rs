use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorForbidden,
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{debug, warn};

use crate::auth::AuthUser;

// Admin guard middleware factory
pub struct AdminGuard;

impl AdminGuard {
    pub fn new() -> Self {
        Self {}
    }
}

// Middleware factory implementation
impl<S, B> Transform<S, ServiceRequest> for AdminGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AdminGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminGuardMiddleware { service }))
    }
}

// Middleware service
pub struct AdminGuardMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AdminGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let path = req.path().to_string(); // Get the request path for logging

        Box::pin(async move {
            debug!("Checking admin role for path: {}", path);

            // Get the authenticated user from request extensions
            // This is set by the JwtAuth middleware which should run before this middleware
            let auth_user = req.extensions().get::<AuthUser>().cloned();

            match auth_user {
                Some(user) => {
                    // Check if the user has the admin role
                    if user.role.to_lowercase() == "admin" {
                        debug!("Admin access granted for user {} to path: {}", user.id, path);
                        service.call(req).await
                    } else {
                        warn!("Admin access denied for user {} with role {} to path: {}", 
                              user.id, user.role, path);
                        Err(ErrorForbidden("Admin role required for this resource"))
                    }
                }
                None => {
                    warn!("Admin access denied: No authenticated user found for path: {}", path);
                    Err(ErrorForbidden("Authentication required"))
                }
            }
        })
    }
}