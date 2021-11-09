use std::str::Utf8Error;

use crate::http::StatusCode;

/// Represents a header field value as defined in [RFC7230 Section
/// 3.2](https://datatracker.ietf.org/doc/html/rfc7230#section-3.2).
///
/// FieldValue cannot always be represented as a valid [`String`] as
/// the `obs-text` allowed in these values are not US-ASCII so it we
/// cannot guarentee that it will be valid UTF-8. The FieldValue will
/// therefore be stored as bytes and can be compared to [`str`]s.
///
/// ```text
/// field-value = *( field-content / obs-fold )
///
/// field-content = field-vchar [ 1*(SP / HTAB) field-vchar ]
/// field-vchar = VCHAR / obs-text
/// VCHAR = %x21-7E; visible characters
/// obs-text = %x80-FF; end of US-ASCII to u8::MAX
///
/// obs-fold = CRLF 1*( SP / HTAB )
/// ```
#[derive(Debug, PartialEq)]
pub struct HeaderFieldValue(Vec<u8>);

impl HeaderFieldValue {
    /// Derive a [`HeaderFieldValue`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of [`HeaderFieldValue`].
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        Ok(Self(src.to_vec()))
    }

    /// Returns a [`str`] if the header field value contains visible ASCII characters.
    pub fn try_as_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}
