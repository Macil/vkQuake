#![forbid(unsafe_code)]

use http::{header, Request, Response, StatusCode};
use http_body::Body;
use std::{fmt, marker::PhantomData};
use tower_http::validate_request::ValidateRequest;

pub struct CsrfProtection<ResBody> {
    external_allowed_origins: Vec<String>,
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> CsrfProtection<ResBody> {
    pub fn new() -> Self
    where
        ResBody: Body + Default,
    {
        Self::with_external_allowed_origins(vec![])
    }

    pub fn with_external_allowed_origins(extra_allowed_origins: Vec<String>) -> Self
    where
        ResBody: Body + Default,
    {
        Self {
            external_allowed_origins: extra_allowed_origins,
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for CsrfProtection<ResBody> {
    fn clone(&self) -> Self {
        Self {
            external_allowed_origins: self.external_allowed_origins.clone(),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for CsrfProtection<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CsrfProtection")
            .field("extra_allowed_hosts", &self.external_allowed_origins)
            .finish()
    }
}

impl<B, ResBody> ValidateRequest<B> for CsrfProtection<ResBody>
where
    ResBody: Body + Default,
{
    type ResponseBody = ResBody;

    fn validate(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        // CSRF only applies to methods other than GET, HEAD, or OPTIONS.
        if request.method() == http::Method::GET
            || request.method() == http::Method::HEAD
            || request.method() == http::Method::OPTIONS
        {
            return Ok(());
        }

        if let Some(Ok(origin_value)) = request.headers().get(header::ORIGIN).map(|h| h.to_str()) {
            // Check if the Origin is in the list of allowed origins.
            if self
                .external_allowed_origins
                .iter()
                .any(|allowed| allowed == origin_value)
            {
                return Ok(());
            }

            // Otherwise check if the Origin matches the Host header.
            if let Some(Ok(host_value)) = request.headers().get(header::HOST).map(|h| h.to_str()) {
                // Origin will be a protocol and a hostname. Check that the protocol is
                // http or https, and that the hostname matches the Host header.
                let mut origin_parts = origin_value.split("://");
                let origin_proto = origin_parts.next();
                if origin_proto == Some("http") || origin_proto == Some("https") {
                    let origin_host = origin_parts.next();
                    if origin_host == Some(host_value) && origin_parts.next().is_none() {
                        return Ok(());
                    }
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
    async fn valid_get() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::GET)
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_match_http() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .header(header::ORIGIN, "http://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_match_https() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .header(header::ORIGIN, "https://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_missing_origin() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn invalid_protocol() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .header(header::ORIGIN, "httpx://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn invalid_match() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .header(header::ORIGIN, "https://localhost:4000")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn invalid_match_repeated_separator() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(CsrfProtection::new()))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::HOST, "localhost:3000")
            .header(header::ORIGIN, "https://localhost:3000://")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn valid_extra_allowed_origin() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                CsrfProtection::with_external_allowed_origins(vec![
                    "http://foo.example.com".to_string(),
                    "https://example.com".to_string(),
                ]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::ORIGIN, "https://example.com")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_with_extra_allowed_origins() {
        let mut service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::custom(
                CsrfProtection::with_external_allowed_origins(vec![
                    "http://foo.example.com".to_string(),
                    "https://example.com".to_string(),
                ]),
            ))
            .service_fn(echo);

        let request = Request::get("/")
            .method(http::Method::POST)
            .header(header::ORIGIN, "https://example.com2")
            .body(Body::empty())
            .unwrap();

        let res = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    async fn echo(req: Request<Body>) -> Result<Response<Body>, BoxError> {
        Ok(Response::new(req.into_body()))
    }
}
