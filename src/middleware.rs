use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::sync::Arc;
use actix_web::dev::{ResourcePath, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse, web};
use std::task::{Context, Poll};
use actix_web::body::BoxBody;
use crate::{AppData, server::{CONTRACTS_PATH, RECOVER_PROGRESS_PATH}};
use crate::response::{ExodusResponse, ExodusStatus};

pub struct CompletedStateCheck;

impl<S, B> Transform<S, ServiceRequest> for CompletedStateCheck
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Transform = RecoverStateCheckMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RecoverStateCheckMiddleware { service }))
    }
}

pub struct RecoverStateCheckMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RecoverStateCheckMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let data = req.app_data::<web::Data<Arc<AppData>>>().unwrap();

        match req.path(){
            RECOVER_PROGRESS_PATH | CONTRACTS_PATH => Box::pin(self.service.call(req)),
            _ => if data.is_not_sync_completed(){
                Box::pin(async move {
                    let response: ExodusResponse<()> = ExodusStatus::RecoverStateUnfinished.into();
                    Ok(req.into_response(HttpResponse::Ok().json(response)))
                })
            } else {
                Box::pin(self.service.call(req))
            }
        }
    }
}
