use std::num::NonZeroU16;

/// An HTTP Status Code representation as defined in the RFC (RFC 7231 Section
/// 6)[https://datatracker.ietf.org/doc/html/rfc7231#section-6]
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct StatusCode(NonZeroU16);

/// An Error type to signal that a conversion failed
#[derive(Debug)]
pub struct InvalidStatusCode;

macro_rules! const_status_codes {
    (
        $(
            $(#[$comment:meta])+
            $name:ident => $code:literal, $reason:literal,
        )*
    ) => {
        impl StatusCode {
            $(
                $(#[$comment])+
                pub const $name: StatusCode = StatusCode(unsafe { NonZeroU16::new_unchecked($code) });
            )*

            pub const fn reason(&self) -> &'static str {
                match self.0.get() {
                    $(
                        $code => $reason,
                    )*
                    // StatusCode valid instances are defined at compile time and so
                    // the u16 must match on one of the codes used to define a valid instance
                    // Note: unreachable & panic are not stable
                    _ => "Unreachable"
                }
            }

            pub fn from_bytes(src: &[u8]) -> Result<Self, InvalidStatusCode> {
                if let [a @ b'1'..=b'9', b @ b'0'..=b'9', c @ b'0'..=b'9'] = src {
                    let a = a.wrapping_sub(b'0') as u16;
                    let b = b.wrapping_sub(b'0') as u16;
                    let c = c.wrapping_sub(b'0') as u16;

                    let code = (a * 100) + (b * 10) + c;
                    match code {
                        $(
                            $code => return Ok(Self::$name),
                        )*
                        _ => {},
                    }
                }

                Err(InvalidStatusCode)
            }
        }

    }
}

const_status_codes! {
    /// 100 Continue
    /// Informational code as defined in [RFC 7231 Section
    /// 6.2.1](https://datatracker.ietf.org/doc/html/rfc7231#section-6.2.1)
    CONTINUE => 100, "Continue",
    /// 101 Switching Protocols
    /// Informational code as defined in [RFC 7231 Section
    /// 6.2.2](https://datatracker.ietf.org/doc/html/rfc7231#section-6.2.2)
    SWITCHING_PROTOCOLS => 101, "Switching Protocols",
    /// 200 OK
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.1](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.1)
    OK => 200, "OK",
    /// 201 Created
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.2](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.2)
    CREATED => 201, "Created",
    /// 202 Accepted
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.3](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.3)
    ACCEPTED => 202, "Accepted",
    /// 203 Non-Authoritative Information
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.3](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.3)
    NON_AUTHRITATIVE_INFORMATION => 203, "Non-Authoritative Information",
    /// 204 No Content
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.4](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.4)
    NO_CONTENT => 204, "No Content",
    /// 205 Reset Content
    /// Successful code as defined in [RFC 7231 Section
    /// 6.3.5](https://datatracker.ietf.org/doc/html/rfc7231#section-6.3.5)
    RESET_CONTENT => 205, "Reset Content",
    /// 206 Partial Content
    /// Successful code as defined in [RFC 7233 Section
    /// 4.1](https://datatracker.ietf.org/doc/html/rfc7233#section-4.1)
    PARTIAL_CONTENT => 206, "Partial Content",
    /// 300 Multiple Choices
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.1](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.1)
    MULTIPLE_CHOICES => 300, "Multiple Choices",
    /// 301 Moved Permanently
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.2](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.2)
    MOVED_PERMANENTLY => 301, "Moved Permanently",
    /// 302 Found
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.3](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.3)
    FOUND => 302, "Found",
    /// 303 See Other
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.4](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.4)
    SEE_OTHER => 303, "See Other",
    /// 304 Not Modified
    /// Redirection code as defined in [RFC 7232 Section
    /// 4.1](https://datatracker.ietf.org/doc/html/rfc7232#section-4.1)
    NOT_MODIFIED => 304, "Not Modified",
    /// 305 Use Proxy
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.5](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.5)
    USE_PROXY => 305, "Use Proxy",
    /// 307 Temporary Redirect
    /// Redirection code as defined in [RFC 7231 Section
    /// 6.4.7](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4.7)
    TEMPORARY_REDIRECT => 307, "Temporary Redirect",
    /// 400 Bad Request
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.1](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.1)
    BAD_REQUEST => 400, "Bad Request",
    /// 401 Unauthorized
    /// Client Error code as defined in [RFC 7235 Section
    /// 3.1](https://datatracker.ietf.org/doc/html/rfc7235#section-3.1)
    UNAUTHORIZED => 401, "Unauthorized",
    /// 402 Payment Required
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.2](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.2)
    PAYMENT_REQUIRED => 402, "Payment Required",
    /// 403 Forbidden
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.3](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.3)
    FORBIDDEN => 403, "Forbidden",
    /// 404 Not Found
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.4](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.4)
    NOT_FOUND => 404, "Not Found",
    /// 405 Method Not Allowed
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.5](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.5)
    METHOD_NOT_ALLOWED => 405, "Method Not Allowed",
    /// 406 Not Acceptable
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.6](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.6)
    NOT_ACCEPTABLE => 406, "Not Acceptable",
    /// 407 Proxy Authentication Required
    /// Client Error code as defined in [RFC 7235 Section
    /// 3.2](https://datatracker.ietf.org/doc/html/rfc7235#section-3.2)
    PROXY_AUTHENTICATION_REQUIRED => 407, "Proxy Authentication Required",
    /// 408 Request Timeout
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.7](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.7)
    REQUEST_TIMEOUT => 408, "Request Timeout",
    /// 409 Conflict
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.8](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.8)
    CONFLICT => 409, "Conflict",
    /// 410 Gone
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.9](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.9)
    GONE => 410, "Gone",
    /// 411 Length Required
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.10](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.10)
    LENGTH_REQUIRED => 411, "Length Required",
    /// 412 Precondition Failed
    /// Client Error code as defined in [RFC 7232 Section
    /// 4.2](https://datatracker.ietf.org/doc/html/rfc7232#section-4.2)
    PRECONDITION_FAILED => 412, "Precondition Failed",
    /// 413 Payload Too Large
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.11](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.11)
    PAYLOAD_TOO_LARGE => 413, "Payload Too Large",
    /// 414 URI Too Long
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.12](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.12)
    URI_TOO_LONG => 414, "URI Too Long",
    /// 415 Unsupported Media Type
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.13](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.13)
    UNSUPPORTED_MEDIA_TYPE => 415, "Unsupported Media Type",
    /// 416 Range Not Satisfiable
    /// Client Error code as defined in [RFC 7233 Section
    /// 4.4](https://datatracker.ietf.org/doc/html/rfc7233#section-4.4)
    RANGE_NOT_SATISFIABLE => 416, "Range Not Satisfiable",
    /// 417 Expectation Failed
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.14](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.14)
    EXPECTATION_FAILED => 417, "Expectation Failed",
    /// 426 Upgrade Required
    /// Client Error code as defined in [RFC 7231 Section
    /// 6.5.15](https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.15)
    UPGRADE_REQUIRED => 426, "Upgrade Required",
    /// 500 Internal Server Error
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.1](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.1)
    INTERNAL_SERVER_ERROR => 500, "Internal Server Error",
    /// 501 Not Implemented
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.2](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.2)
    NOT_IMPLEMENTED => 501, "Not Implemented",
    /// 502 Bad Gateway
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.3](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.3)
    BAD_GATEWAY => 502, "Bad Gateway",
    /// 503 Service Unavailable
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.4](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.4)
    SERVICE_UNAVAILABLE => 503, "Service Unavailable",
    /// 504 Gateway Timeout
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.5](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.5)
    GATEWAY_TIMEOUT => 504, "Gateway Timeout",
    /// 505 HTTP Version Not Supported
    /// Server Error code as defined in [RFC 7231 Section
    /// 6.6.6](https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.6)
    HTTP_VERSION_NOT_SUPPORTED => 505, "HTTP Version Not Supported",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_three_digit_ascii_is_status_code() {
        assert_eq!(StatusCode::OK, StatusCode::from_bytes(b"200").unwrap());
        assert_eq!(
            StatusCode::NOT_FOUND,
            StatusCode::from_bytes(b"404").unwrap()
        );
        assert_eq!(
            StatusCode::NOT_IMPLEMENTED,
            StatusCode::from_bytes(b"501").unwrap()
        );
    }

    #[test]
    fn not_enough_bytes_is_an_invalid_status_code() {
        assert!(StatusCode::from_bytes(b"2").is_err());
        assert!(StatusCode::from_bytes(b"20").is_err());
    }

    #[test]
    fn too_much_bytes_is_an_invalid_status_code() {
        assert!(StatusCode::from_bytes(b"1000").is_err());
        assert!(StatusCode::from_bytes(b"10000").is_err());
    }

    #[test]
    fn unknown_three_ascii_digits_is_an_invalid_status_code() {
        assert!(StatusCode::from_bytes(b"000").is_err());
        assert!(StatusCode::from_bytes(b"190").is_err());
        assert!(StatusCode::from_bytes(b"999").is_err());
    }
}
