#![forbid(unsafe_code)]

use http::{header, Request, Response, StatusCode};
use http_body::Body;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{fmt, marker::PhantomData};
use tower_http::validate_request::ValidateRequest;

// Matches a host header that uses an IP address instead of a domain name.
// This check is a little loose at checking validity of IP addresses; we're just
// looking for something that's obviously not a valid DNS name.
static HOST_IP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(\d+\.\d+\.\d+\.\d+|\[[:\da-f]+\])(:\d+)?$").unwrap());

pub struct RebindingProtection<ResBody> {
    allowed_hosts: Vec<String>,
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> RebindingProtection<ResBody> {
    pub fn new(allowed_hosts: Vec<String>) -> Self
    where
        ResBody: Body + Default,
    {
        Self {
            allowed_hosts,
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for RebindingProtection<ResBody> {
    fn clone(&self) -> Self {
        Self {
            allowed_hosts: self.allowed_hosts.clone(),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for RebindingProtection<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RebindingProtection")
            .field("allowed_hosts", &self.allowed_hosts)
            .finish()
    }
}

impl<B, ResBody> ValidateRequest<B> for RebindingProtection<ResBody>
where
    ResBody: Body + Default,
{
    type ResponseBody = ResBody;

    fn validate(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        if let Some(host) = request.headers().get(header::HOST) {
            if let Ok(host_str) = host.to_str() {
                // Allow the request if the host header is an IP address, because a DNS rebinding
                // attack can't be done with an IP address.
                if HOST_IP_RE.is_match(host_str) {
                    return Ok(());
                }

                // Allow the request if the host (ignoring port) is in the list of allowed hosts.
                let host_without_port = host_str.split(':').next().unwrap();
                if self.allowed_hosts.iter().any(|x| x == host_without_port) {
                    return Ok(());
                }
            }
        }

        let mut res = Response::new(ResBody::default());
        *res.status_mut() = StatusCode::FORBIDDEN;
        Err(res)
    }
}

#[cfg(test)]
mod tests {
    use tower_http::validate_request::ValidateRequestHeaderLayer;

    use super::*;
    use http::header;
    use hyper::Body;
    use tower::{BoxError, ServiceBuilder, ServiceExt};
    use tower_service::Service;

    #[tokio::test]
    async fn allowed_host() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                RebindingProtection::new(vec!["localhost".to_string()]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::HOST, "localhost:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ipv4_address() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                RebindingProtection::new(vec!["localhost".to_string()]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::HOST, "127.0.0.1:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ipv6_address() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                RebindingProtection::new(vec!["localhost".to_string()]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::HOST, "[::1]:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_host() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                RebindingProtection::new(vec!["localhost".to_string()]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::HOST, "evil.example.com:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn missing_host() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                RebindingProtection::new(vec!["localhost".to_string()]),
            ))
            .service_fn(echo);

        let request = Request::get("/").body(Body::empty()).unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    async fn echo(req: Request<Body>) -> Result<Response<Body>, BoxError> {
        Ok(Response::new(req.into_body()))
    }
}
