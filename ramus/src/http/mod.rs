mod method;
mod request;
mod status_code;
mod uri;
pub(crate) mod utils;

pub use request::*;
pub use status_code::*;
pub use uri::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Version((u8, u8));

impl Version {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        if let Some([major @ b'0'..=b'9', b'.', minor @ b'0'..=b'9']) = src.strip_prefix(b"HTTP/") {
            Ok(Version((*major, *minor)))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

impl Version {
    pub fn major(&self) -> u8 {
        self.0 .0 - b'0'
    }

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
