mod file_server;
mod proxy;

use anyhow::Result;
use futures::Future;
use http::{Request, Response};
use hyper::Body;
use std::convert::TryFrom;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::addon::proxy::Proxy;
use crate::Config;

use super::middleware::Middleware;

use self::proxy::ProxyHandler;

/// The main handler for the HTTP request, a HTTP response is created
/// as a result of this handler.
///
/// This handler will be executed against the HTTP request after every
/// "Middleware Before" chain is executed but before any "Middleware After"
/// chain is executed
pub type Handler = Box<
    dyn Fn(
            Arc<Mutex<Request<Body>>>,
        ) -> Pin<Box<dyn Future<Output = http::Response<Body>> + Send + Sync>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct HttpHandler {
    proxy_handler: Arc<ProxyHandler>,
    middleware: Arc<Middleware>,
}

impl HttpHandler {
    pub async fn handle_request(self, request: Request<Body>) -> Result<Response<Body>> {
        let handler = Arc::clone(&self.proxy_handler);
        let middleware = Arc::clone(&self.middleware);
        let response = middleware.handle(request, handler.handle()).await;

        Ok(response)
    }
}

impl From<Arc<Config>> for HttpHandler {
    fn from(config: Arc<Config>) -> Self {
        let proxy = Proxy::new("https://estebanborai.com");
        let proxy_handler = Arc::new(ProxyHandler::new(proxy));
        let middleware = Middleware::try_from(config).unwrap();
        let middleware = Arc::new(middleware);

        HttpHandler {
            proxy_handler,
            middleware,
        }
    }
}
