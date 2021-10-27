use crate::http::StatusCode;

/// Scheme of the URI as defined in [RFC3986 Section
/// 3.1](https://datatracker.ietf.org/doc/html/rfc3986#section-3.1).
///
/// ```text
/// scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scheme(String);

impl Scheme {
    /// Create a Scheme from a sequence of bytes.
    ///
    /// An empty slice will return a valid Scheme which is empty.
    ///
    /// # Errors
    /// If not empty the first ascii char MUST be a ALPHA otherwise this will
    /// return an [`Err(StatusCode::BAD_REQUEST)`]
    ///
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        if src.is_empty() {
            return Ok(Self(String::new()));
        }
        if !src[0].is_ascii_alphabetic() {
            return Err(StatusCode::BAD_REQUEST);
        }

        if src.iter().skip(1).all(|b| {
            matches!(b,
                b'A'..=b'Z' |
                b'a'..=b'z' |
                b'+' | b'-' | b'.'
            )
        }) {
            // SAFETY:
            // src is known to be a valid (restricted) ASCII sequence which
            // doesn't need to be checked for invalid sequences for utf8.
            Ok(Self(unsafe {
                String::from_utf8_unchecked(src.to_ascii_lowercase())
            }))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }

    /// Returns true if the Scheme is empty (has no value)
    ///
    /// ```
    /// use ramus::http::Scheme;
    ///
    /// let scheme = Scheme::default();
    /// assert!(scheme.is_empty())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod scheme_tests {
    use super::*;

    fn assert_is_a_bad_request(bytes: &[u8]) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), Scheme::from_bytes(bytes));
    }

    #[test]
    fn non_alpha_prefix_ascii_char_is_a_bad_request() {
        assert_is_a_bad_request(b"+");
        assert_is_a_bad_request(b"1");
        assert_is_a_bad_request(b"%");
    }

    #[test]
    fn invalid_ascii_in_scheme_is_a_bad_request() {
        assert_is_a_bad_request(b"http~");
        assert_is_a_bad_request(b"c@t");
    }

    #[test]
    fn empty_scheme_is_valid() {
        assert_eq!(Ok(Scheme(String::new())), Scheme::from_bytes(&[]));
    }

    #[test]
    fn known_protocols_are_schemes() {
        assert_eq!(Ok(Scheme("http".to_owned())), Scheme::from_bytes(b"http"));
        assert_eq!(Ok(Scheme("https".to_owned())), Scheme::from_bytes(b"https"));
    }

    #[test]
    fn scheme_will_normalize_to_lowercase() {
        assert_eq!(Ok(Scheme("http".to_owned())), Scheme::from_bytes(b"HTTP"));
        assert_eq!(
            Ok(Scheme("scheme".to_owned())),
            Scheme::from_bytes(b"SCHEME")
        );
    }
}
