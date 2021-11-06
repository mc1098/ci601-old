use super::StatusCode;

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
#[non_exhaustive]
/// Request methods as defined in [RFC7231 Section
/// 4](https://datatracker.ietf.org/doc/html/rfc7231#section-4)
pub enum Method {
    /// Transfer a current representation of the target resource.
    Get,
    /// Same as [`Method::Get`], but only transfer the status line and
    /// header section.
    Head,
    /// Perform resource-specific processing on the request payload.
    Post,
    /// Replace all current representations of the target resource with the request payload.
    Put,
    /// Remove all current representations of the target resource.
    Delete,
    /// Establish a tunnel to the server identified by the target resource.
    Connect,
    /// Describe the communication options for the target resource.
    Options,
    /// Perform a message loop-back test along the path to the target resource.
    Trace,
}

impl Method {
    /// Derive a [`Method`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of [`Method`].
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
