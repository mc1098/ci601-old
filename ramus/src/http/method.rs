use super::StatusCode;

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
#[non_exhaustive]
/// Request methods as defined in [RFC7231 Section
/// 4](https://datatracker.ietf.org/doc/html/rfc7231#section-4)
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl Method {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        let method = match src {
            b"GET" => Method::Get,
            b"HEAD" => Method::Head,
            b"POST" => Method::Post,
            b"PUT" => Method::Put,
            b"DELETE" => Method::Delete,
            b"CONNECT" => Method::Connect,
            b"OPTIONS" => Method::Options,
            b"TRACE" => Method::Trace,
            _ => return Err(StatusCode::NOT_IMPLEMENTED),
        };
        Ok(method)
    }
}
