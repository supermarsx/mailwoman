//! Bounded HTTP admission and ingress deadlines; response streams have no idle deadline.
use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode, Version, header},
    response::IntoResponse,
    serve::Listener,
};
use bytes::Bytes;
use futures_util::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use hyper::{
    body::{Body as HttpBody, Frame, Incoming, SizeHint},
    service::{Service, service_fn},
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo, TokioTimer},
    server::{conn::auto::Builder, graceful::GracefulShutdown},
    service::TowerToHyperService,
};
use std::{
    convert::Infallible,
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{OwnedSemaphorePermit, Semaphore, oneshot},
    task::JoinSet,
    time::Sleep,
};
#[path = "http_limits.rs"]
pub mod limits;
pub use limits::HttpLimits;
type Cancellation = Shared<BoxFuture<'static, ()>>;

/// Serve the configured router, retaining trusted peer identity and HTTP upgrades.
pub async fn serve<L, F>(
    mut listener: L,
    router: Router,
    limits: HttpLimits,
    shutdown: F,
) -> io::Result<()>
where
    L: Listener<Addr = SocketAddr> + Send + 'static,
    L::Io: Send + 'static,
    F: Future<Output = ()> + Send,
{
    limits.validate()?;
    let connections = Arc::new(Semaphore::new(limits.connections));
    let requests = Arc::new(Semaphore::new(limits.requests));
    let graceful = GracefulShutdown::new();
    let mut tasks = JoinSet::new();
    let (_cancel_tx, cancel_rx) = oneshot::channel::<()>();
    let cancel = async move {
        let _ = cancel_rx.await;
    }
    .boxed()
    .shared();
    tokio::pin!(shutdown);
    loop {
        let (io, peer) = tokio::select! {
            biased;
            _=&mut shutdown=>break,
            _=tasks.join_next(),if !tasks.is_empty()=>continue,
            connection=listener.accept()=>connection,
        };
        // Keep polling the listener even when HTTP capacity is full, so pending
        // TLS deadlines remain live. Refuse the excess socket before any handler.
        let Ok(permit) = connections.clone().try_acquire_owned() else {
            drop(io);
            continue;
        };
        let first_request = Arc::new(AtomicBool::new(false));
        let io = ConnectionIo {
            inner: io,
            _permit: permit,
            cancel_read: cancel.clone(),
            cancel_write: cancel.clone(),
            first_request: first_request.clone(),
            first_deadline: Box::pin(tokio::time::sleep(limits.header_timeout)),
        };
        let app = router.clone();
        let requests = requests.clone();
        let watcher = graceful.watcher();
        tasks.spawn(async move {
            let service = service_fn(move |request: Request<Incoming>| {
                let app = app.clone();
                let requests = requests.clone();
                let first_request = first_request.clone();
                async move {
                    first_request.store(true, Ordering::Release);
                    let version = request.version();
                    let header_bytes = request.headers().iter().fold(
                        request.uri().to_string().len() + request.method().as_str().len() + 12,
                        |sum, (key, value)| {
                            sum.saturating_add(key.as_str().len() + value.len() + 4)
                        },
                    );
                    if request.headers().len() > limits.headers
                        || header_bytes > limits.header_bytes
                    {
                        return Ok::<_, Infallible>(refusal(
                            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
                            "Request headers exceed configured limits",
                            version,
                        ));
                    }
                    let Ok(permit) = requests.try_acquire_owned() else {
                        let mut response = refusal(
                            StatusCode::SERVICE_UNAVAILABLE,
                            "Server request capacity reached",
                            version,
                        );
                        response
                            .headers_mut()
                            .insert(header::RETRY_AFTER, header::HeaderValue::from_static("1"));
                        return Ok(response);
                    };
                    let (mut parts, body) = request.into_parts();
                    parts.extensions.insert(ConnectInfo(peer));
                    let complete = Arc::new(AtomicBool::new(body.is_end_stream()));
                    let expired = Arc::new(AtomicBool::new(false));
                    let body = IngressBody {
                        inner: body,
                        deadline: Box::pin(tokio::time::sleep(limits.body_timeout)),
                        complete: complete.clone(),
                        expired: expired.clone(),
                        finished: false,
                    };
                    let service = TowerToHyperService::new(app);
                    let mut response = service
                        .call(Request::from_parts(parts, Body::new(body)))
                        .await?;
                    if expired.load(Ordering::Acquire) {
                        response = refusal(
                            StatusCode::REQUEST_TIMEOUT,
                            "Request body timed out",
                            version,
                        );
                    }
                    if version != Version::HTTP_2 && !complete.load(Ordering::Acquire) {
                        // Do not let Hyper drain an abandoned slow HTTP1 body
                        // outside the ingress deadline wrapper.
                        if response.status() == StatusCode::SWITCHING_PROTOCOLS {
                            response = refusal(
                                StatusCode::BAD_REQUEST,
                                "Upgrade request body must be empty",
                                version,
                            );
                        }
                        response.headers_mut().insert(
                            header::CONNECTION,
                            header::HeaderValue::from_static("close"),
                        );
                    }
                    Ok(response.map(|body| {
                        Body::new(ResponseLease {
                            inner: body,
                            _permit: permit,
                        })
                    }))
                }
            });
            let mut builder = Builder::new(TokioExecutor::new());
            builder
                .http1()
                .timer(TokioTimer::new())
                .header_read_timeout(limits.header_timeout)
                .max_headers(limits.headers)
                .max_buf_size(limits.header_bytes);
            builder
                .http2()
                .timer(TokioTimer::new())
                .max_concurrent_streams(limits.http2_streams)
                .max_header_list_size(limits.header_bytes as u32)
                .adaptive_window(false)
                .initial_stream_window_size(65535)
                .initial_connection_window_size(1024 * 1024)
                .max_send_buf_size(65536)
                .keep_alive_interval(std::time::Duration::from_secs(15))
                .keep_alive_timeout(std::time::Duration::from_secs(10))
                .enable_connect_protocol();
            let connection = builder.serve_connection_with_upgrades(TokioIo::new(io), service);
            if watcher.watch(connection).await.is_err() {
                tracing::debug!("HTTP connection ended with a protocol or transport error");
            }
        });
    }
    drop(listener);
    let _ = tokio::time::timeout(limits.shutdown_timeout, graceful.shutdown()).await;
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    Ok(())
}
fn refusal(status: StatusCode, message: &'static str, version: Version) -> Response<Body> {
    let mut response = (status, message).into_response();
    if version != Version::HTTP_2 {
        response.headers_mut().insert(
            header::CONNECTION,
            header::HeaderValue::from_static("close"),
        );
    }
    response
}

struct ConnectionIo<I> {
    inner: I,
    _permit: OwnedSemaphorePermit,
    cancel_read: Cancellation,
    cancel_write: Cancellation,
    first_request: Arc<AtomicBool>,
    first_deadline: Pin<Box<Sleep>>,
}
impl<I: AsyncRead + Unpin> AsyncRead for ConnectionIo<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if Pin::new(&mut this.cancel_read).poll(cx).is_ready() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "server stopped",
            )));
        }
        if !this.first_request.load(Ordering::Acquire)
            && this.first_deadline.as_mut().poll(cx).is_ready()
        {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "HTTP opening deadline elapsed",
            )));
        }
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}
impl<I: AsyncWrite + Unpin> AsyncWrite for ConnectionIo<I> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if Pin::new(&mut this.cancel_write).poll(cx).is_ready() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "server stopped",
            )));
        }
        Pin::new(&mut this.inner).poll_write(cx, bytes)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if Pin::new(&mut this.cancel_write).poll(cx).is_ready() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "server stopped",
            )));
        }
        Pin::new(&mut this.inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if Pin::new(&mut this.cancel_write).poll(cx).is_ready() {
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if Pin::new(&mut this.cancel_write).poll(cx).is_ready() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "server stopped",
            )));
        }
        Pin::new(&mut this.inner).poll_write_vectored(cx, bufs)
    }
}
struct IngressBody {
    inner: Incoming,
    deadline: Pin<Box<Sleep>>,
    complete: Arc<AtomicBool>,
    expired: Arc<AtomicBool>,
    finished: bool,
}
impl HttpBody for IngressBody {
    type Data = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Ready(None);
        }
        if this.inner.is_end_stream() {
            this.finished = true;
            this.complete.store(true, Ordering::Release);
            return Poll::Ready(None);
        }
        if this.deadline.as_mut().poll(cx).is_ready() {
            this.finished = true;
            this.expired.store(true, Ordering::Release);
            crate::request_protection::observe(mw_store::request_protection_model::Signal::BodyDeadline);
            return Poll::Ready(Some(Err(Box::new(io::Error::new(
                io::ErrorKind::TimedOut,
                "request body deadline elapsed",
            )))));
        }
        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                if this.inner.is_end_stream() {
                    this.complete.store(true, Ordering::Release);
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Ready(Some(Err(error))) => {
                this.finished = true;
                Poll::Ready(Some(Err(Box::new(error))))
            }
            Poll::Ready(None) => {
                this.finished = true;
                this.complete.store(true, Ordering::Release);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
    fn is_end_stream(&self) -> bool {
        self.finished || self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
struct ResponseLease {
    inner: Body,
    _permit: OwnedSemaphorePermit,
}
impl HttpBody for ResponseLease {
    type Data = Bytes;
    type Error = axum::Error;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Pin::new(&mut self.get_mut().inner).poll_frame(cx)
    }
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
