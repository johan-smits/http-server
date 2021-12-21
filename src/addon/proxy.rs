use http::header::{HeaderName, HeaderValue};
use hyper::Body;

use crate::server::middleware::Request;

pub struct Proxy;

impl Proxy {
    pub fn new() -> Self {
        Proxy
    }

    /// Creates a `Via` HTTP header for the provided HTTP Request.
    ///
    /// The Via general header is added by proxies, both forward and reverse, and
    /// can appear in the request or response headers. It is used for tracking
    /// message forwards, avoiding request loops, and identifying the protocol
    /// capabilities of senders along the request/response chain.
    ///
    /// Via: [ <protocol-name> "/" ] <protocol-version> <host> [ ":" <port> ]
    ///
    /// Refer: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Via
    async fn add_via_header(&self, request: Request<Body>) {
        let mut request = request.lock().await;
        let via_header_str = format!("{:?} Rust http-server", request.version());
        let via_header = HeaderValue::from_str(&via_header_str).unwrap();

        if let Some(current_via_header) = request.headers().get("via") {
            let current_via_header = current_via_header.to_str().unwrap();

            if current_via_header.contains(&via_header_str) {
                return;
            }

            let mut via_set = current_via_header.split(',').collect::<Vec<&str>>();

            via_set.push(&via_header_str);

            let proxies_list = via_set.join(", ");

            request.headers_mut().remove(HeaderName::from_static("via"));
            request.headers_mut().append(
                HeaderName::from_static("via"),
                HeaderValue::from_str(proxies_list.as_str()).unwrap(),
            );
            return;
        }

        request
            .headers_mut()
            .append(HeaderName::from_static("via"), via_header);
    }
}

mod tests {
    use http::header::{HeaderName, HeaderValue};
    use hyper::Body;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::server::middleware::Request;

    use super::Proxy;

    #[tokio::test]
    async fn adds_via_header_if_not_present() {
        let proxy = Proxy::new();
        let request = http::Request::new(Body::empty());
        let request = Arc::new(Mutex::new(request));

        proxy.add_via_header(Arc::clone(&request)).await;

        let request = request.lock().await;
        let headers = request.headers();

        assert!(headers.get(&HeaderName::from_static("via")).is_some());

        let via_header_value = headers.get(&HeaderName::from_static("via")).unwrap();
        let via_header_value = via_header_value.to_str().unwrap();

        assert_eq!(via_header_value, "HTTP/1.1 Rust http-server");
    }

    #[tokio::test]
    async fn appends_via_header_if_another_is_present() {
        let proxy = Proxy::new();
        let mut request = http::Request::new(Body::empty());
        let headers = request.headers_mut();

        headers.append(
            &HeaderName::from_static("via"),
            HeaderValue::from_str("HTTP/1.1 GoodProxy").unwrap(),
        );

        let request = Arc::new(Mutex::new(request));

        proxy.add_via_header(Arc::clone(&request)).await;

        let request = request.lock().await;
        let headers = request.headers();

        assert!(headers.get(&HeaderName::from_static("via")).is_some());

        let via_header_value = headers.get(&HeaderName::from_static("via")).unwrap();
        let via_header_value = via_header_value.to_str().unwrap();

        assert_eq!(
            via_header_value,
            "HTTP/1.1 GoodProxy, HTTP/1.1 Rust http-server"
        );
    }
}
