//! This module provides type representing the Header section used in HTTP requests and responses.
mod name;
mod value;

use std::{collections::HashMap, ops::Index};

pub use name::*;
pub use value::*;

use crate::http::utils::split_at_next;

use super::{utils, StatusCode};

/// Header map
#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct HeaderMap {
    map: HashMap<HeaderFieldName, HeaderFieldValue>,
    // `Set-Cookie` is an exception to not allowing multiple header fields - so
    // in order to avoid having a map that requires multiple values for only one
    // exception we just have hold extra `Set-Cookie` values here.
    set_cookie_extras: Vec<HeaderFieldValue>,
}

impl HeaderMap {
    /// Creates a new empty [`HeaderMap`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Derive a [`HeaderMap`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of the header section.
    pub fn from_bytes(mut src: &[u8]) -> Result<Self, StatusCode> {
        let mut map = HeaderMap::new();
        while let Some((field, [b'\n', rest @ ..])) = split_at_next(src, b'\r') {
            src = rest;
            let (name, value) = split_at_next(field, b':').ok_or(StatusCode::BAD_REQUEST)?;
            let name = HeaderFieldName::from_bytes(name)?;
            let value = utils::abnf::trim_ows(value);
            let value = HeaderFieldValue::from_bytes(value)?;
            map.insert(name, value);
        }
        Ok(map)
    }

    /// Insert a header field name and field value pair.
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<HeaderFieldValue>
    where
        K: Into<HeaderFieldName>,
        V: Into<HeaderFieldValue>,
    {
        let key = key.into();
        let value = value.into();
        if key == HeaderFieldName::SET_COOKIE {
            self.insert_extra(key, value)
        } else {
            self.map.insert(key, value)
        }
    }

    fn insert_extra(
        &mut self,
        key: HeaderFieldName,
        value: HeaderFieldValue,
    ) -> Option<HeaderFieldValue> {
        if self.map.contains_key(&key) {
            self.set_cookie_extras.push(value);
            None
        } else {
            self.insert(key, value)
        }
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Example
    /// ```
    /// # use ramus::http::header::HeaderMap;
    /// let header = HeaderMap::new();
    /// assert!(header.is_empty())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Index<HeaderFieldName> for HeaderMap {
    type Output = HeaderFieldValue;

    fn index(&self, index: HeaderFieldName) -> &Self::Output {
        &self.map[&index]
    }
}

#[cfg(test)]
mod tests {
    use super::{HeaderFieldName, HeaderFieldValue, HeaderMap};
    use crate::http::StatusCode;

    #[test]
    fn empty_bytes_creates_default_header_map() {
        assert!(HeaderMap::from_bytes(&[])
            .map(|hm| hm.is_empty())
            .unwrap_or_default())
    }

    #[test]
    fn field_name_with_space_before_colon_is_a_bad_request() {
        assert_eq!(
            Err(StatusCode::BAD_REQUEST),
            HeaderMap::from_bytes(b"accept :this will be ignored\r\n")
        );
    }

    #[test]
    fn single_registered_field() {
        let header =
            HeaderMap::from_bytes(b"accept: text/html\r\n").expect("valid header field bytes");
        let value: HeaderFieldValue = "text/html".into();
        assert_eq!(value, header[HeaderFieldName::ACCEPT])
    }
}
