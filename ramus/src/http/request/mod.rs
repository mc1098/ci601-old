use super::{method::Method, utils::split_at_next_space, StatusCode, Uri, Version};

#[derive(Debug, PartialEq)]
pub struct RequestLine {
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestLine {
    pub const URI_MAX_LENGTH: usize = 8000;

    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        let (method_bytes, rest) = split_at_next_space(src).ok_or(StatusCode::BAD_REQUEST)?;
        let method = Method::from_bytes(method_bytes)?;

        let (uri_bytes, rest) = split_at_next_space(rest).ok_or(StatusCode::BAD_REQUEST)?;
        if uri_bytes.len() > Self::URI_MAX_LENGTH {
            return Err(StatusCode::URI_TOO_LONG);
        }
        let uri = Uri::from_bytes(uri_bytes)?;

        // pattern match to assert that version bytes is the end of the array
        // otherwise the request line is not valid
        Ok(Self {
            method,
            uri,
            version: Version::from_bytes(rest)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::http::{method::Method, StatusCode, Uri, Version};

    use super::RequestLine;

    fn assert_is_bad_request(right: Result<RequestLine, StatusCode>) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), right);
    }

    #[test]
    fn empty_is_a_bad_request() {
        assert_is_bad_request(RequestLine::from_bytes(&[]));
    }

    #[test]
    fn no_space_after_method_is_not_implemented_method() {
        // Try to read GET/ as the method which doesn't exist
        // so defaults to 501
        assert_eq!(
            Err(StatusCode::NOT_IMPLEMENTED),
            RequestLine::from_bytes(b"GET/ HTTP/1.1")
        );
    }

    #[test]
    fn no_space_after_uri_is_a_bad_request() {
        assert_is_bad_request(RequestLine::from_bytes(b"GET /HTTP/1.1"));
    }

    #[test]
    fn uri_octets_above_max_length_is_a_uri_too_long() {
        let mut octets = vec![b'G', b'E', b'T', b' '];
        octets.extend([b'u'; 8001]);
        octets.push(b' ');

        assert_eq!(
            Err(StatusCode::URI_TOO_LONG),
            RequestLine::from_bytes(&octets)
        )
    }

    #[test]
    fn simple_request_line_is_valid() {
        let method = Method::from_bytes(b"GET").expect("failed to parse method");
        let uri = Uri::from_bytes(b"/").expect("failed to parse uri");
        let version = Version::from_bytes(b"HTTP/1.1").expect("failed to parse version");
        assert_eq!(
            Ok(RequestLine {
                method,
                uri,
                version,
            }),
            RequestLine::from_bytes(b"GET / HTTP/1.1")
        );
    }
}
