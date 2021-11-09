//! A general purpose module of common HTTP types
mod header;
mod method;
mod request;
mod status_code;
mod uri;
pub(crate) mod utils;

pub use header::*;
pub use method::*;
pub use request::*;
pub use status_code::*;
pub use uri::*;

/// HTTP protocol version as defined in
/// [RFC7230 Section 2.6](https://datatracker.ietf.org/doc/html/rfc7230#section-2.6).
///
/// ```text
/// HTTP-version = HTTP-name "/" DIGIT "." DIGIT
///
/// HTTP-name = %x48.54.54.50 ; "HTTP", case-sensitive
/// DIGIT = 0-9
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Version((u8, u8));

impl Version {
    /// Derive [`Version`] from a slice of bytes.
    ///
    /// Any major and minor version will be accepted, so long the syntax of `HTTP-version` is
    /// followed.
    ///
    /// This funtion will return an [`StatusCode::BAD_REQUEST`] as the error type if the syntax
    /// of the bytes is not valid.
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        if let Some([major @ b'0'..=b'9', b'.', minor @ b'0'..=b'9']) = src.strip_prefix(b"HTTP/") {
            Ok(Version((*major, *minor)))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

impl Version {
    /// Returns the major version number in numerical form.
    ///
    /// # Example
    /// ```
    /// # use ramus::http::Version;
    /// let version = Version::from_bytes(b"HTTP/1.1").expect("valid version bytes");
    /// assert_eq!(1, version.major());
    /// ```
    pub fn major(&self) -> u8 {
        self.0 .0 - b'0'
    }

    /// Returns the minor version number in numerical form.
    ///
    /// # Example
    /// ```
    /// # use ramus::http::Version;
    /// let version = Version::from_bytes(b"HTTP/1.1").expect("valid version bytes");
    /// assert_eq!(1, version.minor());
    /// ```
    pub fn minor(&self) -> u8 {
        self.0 .1 - b'0'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes_is_a_bad_request() {
        assert_eq!(Err(StatusCode::BAD_REQUEST), Version::from_bytes(&[]));
    }

    #[test]
    fn invalid_http_name_is_a_bad_request() {
        assert_eq!(
            Err(StatusCode::BAD_REQUEST),
            Version::from_bytes(b"HHHP/1.1")
        );
    }

    #[test]
    fn invalid_version_numbers_is_a_bad_request() {
        assert_eq!(
            Err(StatusCode::BAD_REQUEST),
            Version::from_bytes(b"HTTP/A.1")
        );
        assert_eq!(
            Err(StatusCode::BAD_REQUEST),
            Version::from_bytes(b"HTTP/1.f")
        );
    }

    #[test]
    fn valid_http_version() {
        assert_eq!(Ok(Version((b'1', b'1'))), Version::from_bytes(b"HTTP/1.1"));
    }
}
