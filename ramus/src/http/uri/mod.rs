mod authority;
mod path;
mod scheme;

pub use authority::*;
pub use path::*;
pub use scheme::*;

use super::{
    utils::{reg_name_ext, split_at_next},
    StatusCode,
};

/// Uniform Resource Identifier (URI) as defined in [RFC7230 Section
/// 2.7](https://datatracker.ietf.org/doc/html/rfc7230#section-2.7)
/// Most of it's components are adopted from
/// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986)
///
/// ```text
/// URI = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
/// hier-part = "//" authority path-abempty
///           / path-absolute
///           / path-rootless
///           / path-empty
/// ```
///
/// Example from [RFC3986 Section 3.4](https://datatracker.ietf.org/doc/html/rfc3986#section-3.4)
/// ```text
/// foo://example.com:8042/over/there?name=ferret#nose
/// \_/   \______________/\_________/ \_________/ \__/
///  |           |             |           |        |
/// scheme    authority      path         query    fragment
///  |   ______________________|_
/// / \ /                        \
/// urn:example:animal:ferret:nose
/// ```
#[derive(Debug, PartialEq)]
pub struct Uri {
    scheme: Scheme,
    authority: Option<Authority>,
    path: Path,
    query: Query,
    fragment: Fragment,
}

impl Uri {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        if src.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let (scheme, rest) = if let Some((bytes, rest)) = split_at_next(src, b':') {
            (Scheme::from_bytes(bytes)?, rest)
        } else {
            (Scheme::default(), src)
        };

        let (authority, rest) = if let [b'/', b'/', rest @ ..] = rest {
            match rest.iter().position(|b| b"/?#".contains(b)) {
                Some(i) => {
                    let authority = Authority::from_bytes(&rest[..i])?;
                    (Some(authority), &rest[i..])
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        } else {
            (None, rest)
        };

        let (path, rest) = match rest.iter().position(|b| b"?#".contains(b)) {
            Some(i) => (Path::from_bytes(&rest[..i])?, &rest[i..]),
            None => {
                return Ok(Self {
                    scheme,
                    authority,
                    path: Path::from_bytes(rest)?,
                    query: Query::default(),
                    fragment: Fragment::default(),
                });
            }
        };

        match rest {
            [b'?'] | [b'#'] | [] => Ok(Self {
                scheme,
                authority,
                path,
                query: Query::default(),
                fragment: Fragment::default(),
            }),
            [b'?', rest @ ..] => match split_at_next(rest, b'#') {
                Some((_, [])) | None => Ok(Self {
                    scheme,
                    authority,
                    path,
                    query: Query::from_bytes(rest)?,
                    fragment: Fragment::default(),
                }),
                Some((query, fragment)) => Ok(Self {
                    scheme,
                    authority,
                    path,
                    query: Query::from_bytes(query)?,
                    fragment: Fragment::from_bytes(fragment)?,
                }),
            },
            [b'#', rest @ ..] => Ok(Self {
                scheme,
                authority,
                path,
                query: Query::default(),
                fragment: Fragment::from_bytes(rest)?,
            }),
            _ => Err(StatusCode::BAD_REQUEST),
        }
    }
}

/// Fragment as defined in [RFC3986 Section
/// 3.5](https://datatracker.ietf.org/doc/html/rfc3986#section-3.5)
///
/// ```text
/// fragment = *( pchar / "/" / "?" )
///
/// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
#[derive(Debug, Default, PartialEq)]
pub struct Fragment(String);

impl Fragment {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        reg_name_ext(src, |b| b"/?".contains(&b))
            .filter(|s| s.len() == src.len())
            .map(Self)
            .ok_or(StatusCode::BAD_REQUEST)
    }

    /// Return true if the Fragment is empty (has no value).
    ///
    /// ```
    /// use ramus::http::Fragment;
    ///
    /// let fragment = Fragment::default();
    /// assert!(fragment.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of chars in the Fragment.
    ///
    /// ```
    /// use ramus::http::Fragment;
    ///
    /// let fragment = Fragment::from_bytes(b"Hello")
    ///     .expect("valid fragment bytes");
    /// assert_eq!(5, fragment.len());
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Query as defined in [RFC3986 Section
/// 3.4](https://datatracker.ietf.org/doc/html/rfc3986#section-3.4)
///
/// ```text
/// query = *( pchar / "/" / "?" )
///
/// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
#[derive(Debug, Default, PartialEq)]
pub struct Query(String);

impl Query {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        reg_name_ext(src, |b| b"/?".contains(&b))
            .filter(|s| s.len() == src.len())
            .map(Self)
            .ok_or(StatusCode::BAD_REQUEST)
    }
}

#[cfg(test)]
mod uri_tests {
    use super::{Authority, Fragment, Path, Query, Scheme, StatusCode, Uri};

    fn assert_is_bad_request(bytes: &[u8]) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), Uri::from_bytes(bytes));
    }

    #[test]
    fn empty_array_is_a_bad_request() {
        assert_is_bad_request(&[]);
    }

    #[test]
    fn authority_without_path_forward_slash_is_a_bad_request() {
        assert_is_bad_request(b"http://example.com")
    }

    #[test]
    fn double_hash_is_a_bad_request() {
        assert_is_bad_request(b"http://example.com/#sss#sh");
    }

    #[test]
    fn single_forward_slash_is_valid() {
        // expect from_bytes of parts to parse correctly
        // as these parts are tested individually
        let scheme = Scheme::from_bytes(b"").expect("failed to parse scheme");
        let authority = None;
        let path = Path::from_bytes(b"/").expect("failed to parse path");
        let query = Query::default();
        let fragment = Fragment::default();
        let uri = Uri {
            scheme,
            authority,
            path,
            query,
            fragment,
        };
        assert_eq!(Ok(uri), Uri::from_bytes(b"/"));
    }

    #[test]
    fn uri_with_scheme_authority_path_query_fragment() {
        // expect from_bytes of parts to parse correctly
        // as these parts are tested individually
        let scheme = Scheme::from_bytes(b"foo").expect("failed to parse scheme");
        let authority =
            Authority::from_bytes(b"example.com:8042").expect("failed to parse authority");
        let path = Path::from_bytes(b"/over/there").expect("failed to parse path");
        let query = Query::from_bytes(b"name=ferret").expect("failed to parse query");
        let fragment = Fragment::from_bytes(b"nose").expect("failed to parse fragment");
        let uri = Uri {
            scheme,
            authority: Some(authority),
            path,
            query,
            fragment,
        };
        assert_eq!(
            Ok(uri),
            Uri::from_bytes(b"foo://example.com:8042/over/there?name=ferret#nose")
        );
    }

    #[test]
    fn scheme_can_be_empty() {
        // expect from_bytes of parts to parse correctly
        // as these parts are tested individually
        let scheme = Scheme::from_bytes(&[]).expect("failed to parse scheme");
        let authority =
            Authority::from_bytes(b"example.com:8042").expect("failed to parse authority");
        let path = Path::from_bytes(b"/over/there").expect("failed to parse path");
        let query = Query::from_bytes(b"name=ferret").expect("failed to parse query");
        let fragment = Fragment::from_bytes(b"nose").expect("failed to parse fragment");
        let uri = Uri {
            scheme,
            authority: Some(authority),
            path,
            query,
            fragment,
        };
        assert_eq!(
            Ok(uri),
            Uri::from_bytes(b"://example.com:8042/over/there?name=ferret#nose")
        );
    }

    #[test]
    fn uri_with_authority_can_have_empty_path() {
        // expect from_bytes of parts to parse correctly
        // as these parts are tested individually
        let scheme = Scheme::from_bytes(b"foo").expect("failed to parse scheme");
        let authority =
            Authority::from_bytes(b"example.com:8042").expect("failed to parse authority");
        let path = Path::default();
        let query = Query::from_bytes(b"name=ferret").expect("failed to parse query");
        let fragment = Fragment::from_bytes(b"nose").expect("failed to parse fragment");
        let uri = Uri {
            scheme,
            authority: Some(authority),
            path,
            query,
            fragment,
        };
        assert_eq!(
            Ok(uri),
            Uri::from_bytes(b"foo://example.com:8042?name=ferret#nose")
        );

        let scheme = Scheme::from_bytes(b"foo").expect("failed to parse scheme");
        let authority =
            Authority::from_bytes(b"example.com:8042").expect("failed to parse authority");
        let path = Path::default();
        let query = Query::default();
        let fragment = Fragment::from_bytes(b"nose").expect("failed to parse fragment");
        let uri = Uri {
            scheme,
            authority: Some(authority),
            path,
            query,
            fragment,
        };
        assert_eq!(Ok(uri), Uri::from_bytes(b"foo://example.com:8042#nose"));
    }

    #[test]
    fn uri_with_authority_and_no_query_or_fragment_must_have_a_forward_slash_path() {
        let scheme = Scheme::from_bytes(b"foo").expect("failed to parse scheme");
        let authority =
            Authority::from_bytes(b"example.com:8042").expect("failed to parse authority");
        let path = Path::from_bytes(b"/").expect("failed to parse path");
        let query = Query::default();
        let fragment = Fragment::default();
        let uri = Uri {
            scheme,
            authority: Some(authority),
            path,
            query,
            fragment,
        };
        assert_eq!(Ok(uri), Uri::from_bytes(b"foo://example.com:8042/"));
    }

    #[test]
    fn authority_is_optional() {
        // expect from_bytes of parts to parse correctly
        // as these parts are tested individually
        let scheme = Scheme::from_bytes(b"foo").expect("failed to parse scheme");
        let authority = None;
        let path = Path::from_bytes(b"/over/there").expect("failed to parse path");
        let query = Query::from_bytes(b"name=ferret").expect("failed to parse query");
        let fragment = Fragment::from_bytes(b"nose").expect("failed to parse fragment");
        let uri = Uri {
            scheme,
            authority,
            path,
            query,
            fragment,
        };
        assert_eq!(
            Ok(uri),
            Uri::from_bytes(b"foo:/over/there?name=ferret#nose")
        );
    }
}
