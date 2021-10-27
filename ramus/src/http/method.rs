use super::StatusCode;

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
#[non_exhaustive]
/// Request methods as defined in [RFC7231 Section
/// 4](https://datatracker.ietf.org/doc/html/rfc7231#section-4)
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
}

impl Method {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        let method = match src {
            b"GET" => Method::GET,
            b"HEAD" => Method::HEAD,
            b"POST" => Method::POST,
            b"PUT" => Method::PUT,
            b"DELETE" => Method::DELETE,
            b"CONNECT" => Method::CONNECT,
            b"OPTIONS" => Method::OPTIONS,
            b"TRACE" => Method::TRACE,
            _ => return Err(StatusCode::NOT_IMPLEMENTED),
        };
        Ok(method)
    }
}
