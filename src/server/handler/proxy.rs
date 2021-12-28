use hyper::{Body, Request};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::addon::proxy::Proxy;

use super::Handler;

pub struct ProxyHandler {
    proxy: Arc<Proxy>,
}

impl ProxyHandler {
    pub fn new(proxy: Proxy) -> Self {
        let proxy = Arc::new(proxy);

        ProxyHandler { proxy }
    }

    pub fn handle(&self) -> Handler {
        let proxy = Arc::clone(&self.proxy);

        Box::new(move |request: Arc<Mutex<Request<Body>>>| {
            let proxy = Arc::clone(&proxy);
            let request = Arc::clone(&request);

            Box::pin(async move { proxy.handle(request).await })
        })
    }
}
