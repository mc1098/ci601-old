mod method;
mod request;
mod status_code;
mod uri;
pub(crate) mod utils;

pub use request::*;
pub use status_code::*;
pub use uri::*;

use crate::http::utils::split_at_next;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Version((u8, u8));

impl Version {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        let rest = if let Some((b"HTTP", rest)) = split_at_next(src, 0x2f) {
            rest
        } else {
            return Err(StatusCode::BAD_REQUEST);
        };

        if let Some(([major], [minor])) = split_at_next(rest, 0x2e) {
            if major.is_ascii_digit() && minor.is_ascii_digit() {
                return Ok(Version((*major, *minor)));
            }
        }
        Err(StatusCode::BAD_REQUEST)
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
