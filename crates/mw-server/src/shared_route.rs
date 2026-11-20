//! Shallow cloning for Axum Route boundaries only, not a general backpressure layer.
//! Axum's Route already reports ready and uses an owned service per request.
//! Keep that behavior while making boxed route clones shallow between layers.
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service, ServiceExt, util::Oneshot};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SharedRouteLayer;
pub(crate) struct SharedRoute<S>(Arc<S>);

/// Wrap one existing root middleware without changing tuple layer ordering.
/// Use only where Axum already provides its always-ready Route boundary.
pub(crate) fn wrap<L>(layer: L) -> (SharedRouteLayer, L) {
    (SharedRouteLayer, layer)
}
impl<S> Clone for SharedRoute<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<S> Layer<S> for SharedRouteLayer {
    type Service = SharedRoute<S>;
    fn layer(&self, service: S) -> Self::Service {
        SharedRoute(Arc::new(service))
    }
}
impl<S, Request> Service<Request> for SharedRoute<S>
where
    S: Service<Request> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Oneshot<S, Request>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // This is exactly Axum Route's existing readiness boundary. Oneshot
        // polls the actual per-request service before invoking its call.
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: Request) -> Self::Future {
        self.0.as_ref().clone().oneshot(request)
    }
}
