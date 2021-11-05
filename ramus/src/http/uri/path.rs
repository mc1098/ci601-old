use crate::http::{utils, StatusCode};

/// Path as defined in [RFC3986 Section
/// 3.3](https://datatracker.ietf.org/doc/html/rfc3986#section-3.3)
///
/// ```text
/// path = path-abempty ; begins with "/" or is empty
///     / path-absolute ; begins with "/" but not "//"
///     / path-noscheme ; begins with a non-colon segment
///     / path-rootless ; begins with a segment
///     / path-empty    ; zero characters
///
/// path-abempty = *( "/" segment )
/// path-absolute = "/" [ segment-nz *( "/" segment ) ]
/// path-noscheme = segment-nz-nc *( "/" segment )
/// path-rootless = segment-nz *( "/" segment )
/// path-empty = 0<pchar>
///
/// segment = *pchar
/// segment-nz = 1*pchar
/// segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
///
/// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
#[derive(Debug, Default, PartialEq)]
pub struct Path(String);

impl Path {
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        macro_rules! parse_pchars_into {
            ($rest:expr) => {{
                let segment_nz = utils::abnf::parse_pchar($rest)
                    .filter(|s| !s.is_empty())
                    .ok_or(StatusCode::BAD_REQUEST)?;
                let rest = &$rest[segment_nz.len()..];
                (segment_nz, rest)
            }};
            (PREFIX, $rest:expr) => {{
                let mut path = String::from("/");
                let (segment_nz, rest) = parse_pchars_into!($rest);
                path.push_str(&segment_nz);
                if !segment_nz.is_empty() {
                    (path, rest)
                } else {
                    (path, &rest[1..])
                }
            }};
        }

        let (mut path, mut rest) = match src {
            [b'/'] => {
                return Ok(Path("/".to_owned()));
            }
            [b'/', rest @ ..] => {
                // path-abempty or path-absolute
                parse_pchars_into!(PREFIX, rest)
            }
            [] => return Ok(Path::default()),
            _ => {
                // path-noscheme or path-rootless
                parse_pchars_into!(src)
            }
        };

        loop {
            match rest {
                [b'/', next @ ..] => {
                    let segment = utils::abnf::parse_pchar(next).ok_or(StatusCode::BAD_REQUEST)?;
                    if segment.is_empty() {
                        if let Some('/') = path.chars().last() {
                            // multiple forward slashes are folded down
                            // into a single forward slash so ignore one
                            // if the last char in path is a forward slash
                        } else {
                            path.push('/');
                        }
                        rest = next;
                    } else {
                        path.push('/');
                        path.push_str(&segment);
                        rest = &rest[1 + segment.len()..];
                    }
                }
                [] => break Ok(Path(path)),
                _ => {
                    break Err(StatusCode::BAD_REQUEST);
                }
            }
        }
    }
}

#[cfg(test)]
mod path_tests {
    use super::{Path, StatusCode};

    fn assert_is_bad_request(bytes: &[u8]) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), Path::from_bytes(bytes));
    }

    #[test]
    fn non_pchar_prefix_is_a_bad_request() {
        assert_is_bad_request(b">hi/yo");
    }

    #[test]
    fn double_forward_slash_is_a_bad_request() {
        assert_is_bad_request(b"//hi")
    }

    #[test]
    fn empty_path_is_valid() {
        assert_eq!(Ok(Path::default()), Path::from_bytes(&[]));
    }

    #[test]
    fn single_forward_slash_is_valid() {
        assert_eq!(Ok(Path("/".into())), Path::from_bytes(b"/"))
    }

    #[test]
    fn multiple_forward_slashes_are_replaced_with_one() {
        assert_eq!(Ok(Path("hi/".to_owned())), Path::from_bytes(b"hi//"))
    }

    #[test]
    fn slash_with_segment_is_valid() {
        assert_eq!(
            Ok(Path("/example".to_owned())),
            Path::from_bytes(b"/example")
        );
    }

    #[test]
    fn slash_with_multiple_segments_is_valid() {
        assert_eq!(
            Ok(Path("/this/is/valid".to_owned())),
            Path::from_bytes(b"/this/is/valid")
        );
    }

    #[test]
    fn segment_nz_then_segment_is_valid() {
        assert_eq!(
            Ok(Path("this:is:@".to_owned())),
            Path::from_bytes(b"this:is:@")
        );
        assert_eq!(
            Ok(Path("this:is:@/".to_owned())),
            Path::from_bytes(b"this:is:@/")
        );
        assert_eq!(
            Ok(Path("this:is:@/valid".to_owned())),
            Path::from_bytes(b"this:is:@/valid")
        );
    }
}
