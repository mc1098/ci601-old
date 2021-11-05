/// Divides one slice into two at the first occurrence of the given element.
///
/// The first slice will contain all elements up to the occurrence of the element (excluding that
/// element itself) and the second will contain all the elements after the occurrence of the
/// element (excluding the element itself).
///
/// If the element given is not contained in the slice then this function will return `None`.
///
pub(crate) fn split_at_next(src: &[u8], byte: u8) -> Option<(&[u8], &[u8])> {
    let split_index = src.iter().position(|b| *b == byte)?;
    if src.len() > split_index + 1 {
        Some((&src[..split_index], &src[split_index + 1..]))
    } else {
        Some((&src[..split_index], &[]))
    }
}

/// A convenience function for [`split_at_next`] with the space (0x40) byte.
pub(crate) fn split_at_next_space(src: &[u8]) -> Option<(&[u8], &[u8])> {
    split_at_next(src, b' ')
}

#[cfg(test)]
mod split_at_next_tests {
    use super::{split_at_next, split_at_next_space};

    #[test]
    fn split_at_first_element_empty_left_and_right_with_rest() {
        let bytes = b"@Hello";
        let (left, right) = split_at_next(bytes, b'@').expect("contains '@'");
        assert!(left.is_empty());
        assert_eq!(&bytes[1..], right);
    }

    #[test]
    fn split_at_last_element_prefix_left_and_empty_right() {
        let bytes = b"spaceattheendb";
        let (left, right) = split_at_next(bytes, b'b').expect("contains 'b'");
        assert_eq!(b"spaceattheend", left);
        assert!(right.is_empty());
    }

    #[test]
    fn excludes_the_element_from_left_and_right() {
        let bytes = b"Hello, World";
        let (left, right) = split_at_next_space(bytes).unwrap();
        assert_eq!(b"Hello,", left);
        assert_eq!(b"World", right);
    }

    #[test]
    fn no_next_element_is_none() {
        assert!(split_at_next(b"baaaaaaaaaaaaaaaaa", b'$').is_none());
    }
}

#[inline]
pub(crate) const fn is_unreserved(byte: u8) -> bool {
    matches!(byte,
        b'A'..=b'Z' |
        b'a'..=b'z' |
        b'0'..=b'9' |
        b'-' | b'.' | b'_' | b'~'
    )
}

pub(crate) fn unreserved_sub_delim_ext<F>(src: &[u8], predicate: F) -> Option<String>
where
    F: Fn(u8) -> bool,
{
    let mut s = String::with_capacity(src.len());
    for b in src {
        match *b {
            b'!'        |
            b'$'..=b',' | // '$', '&', ''', '(', ')', '*', '+', ','
            b';'        |
            b'=' => {
                s.push(*b as char);
            }
            _ if is_unreserved(*b) || predicate(*b) => {
                s.push(*b as char);
            }
            _ => return None,
        }
    }
    Some(s)
}

#[inline]
pub(crate) const fn is_hex_dig(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'A'..=b'F')
}

#[inline]
pub(crate) const fn parse_hex_dig(byte: u8) -> Option<u8> {
    let digit = match byte {
        // b'A' - 10 so that A == 10
        b'A'..=b'F' => (byte - (b'A' - 10)),
        b'0'..=b'9' => (byte - b'0'),
        _ => return None,
    };
    Some(digit)
}

/// Checks that the sequence of octets is a valid reg-name as defined in
/// [RFC3986 Section 3.2.2](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2)
///
/// ```text
/// reg-name = *( unreserved / pct-encoded / sub-delims )
///
/// unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
/// pct-encoded = "%" HEXDIG HEXDIG
/// sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
/// ```
pub(crate) fn reg_name(src: &[u8]) -> Option<String> {
    reg_name_ext(src, |_| false)
}

pub(crate) fn reg_name_ext_until<F>(src: &[u8], predicate: F, until: u8) -> Option<String>
where
    F: Fn(u8) -> bool,
{
    let mut reg_name = String::new();

    let mut i = 0;
    while i < src.len() {
        let byte = *unsafe { src.get_unchecked(i) };
        match byte {
                b'%' => {
                    reg_name.push('%');
                    for _ in 0..2 {
                        i += 1;
                        let digit = src
                            .get(i)
                            .filter(|&&b| is_hex_dig(b))?;
                        reg_name.push(*digit as char);
                    }
                    i += 1;
                }
                b'!'        |
                b'$'..=b',' | // '$', '&', ''', '(', ')', '*', '+', ','
                b';'        |
                b'=' => {
                    reg_name.push(byte as char);
                    i += 1;
                }
                b if is_unreserved(b) || predicate(b) => {
                    reg_name.push(b as char);
                    i += 1;
                }
                b if b == until => break,
                _ => return None,
            }
    }
    Some(reg_name)
}

/// Checks that the sequence of octets is a valid reg-name or matches the
/// predicate given.
///
/// See [`reg_name`] for version of this function without predicate
pub(crate) fn reg_name_ext<F>(src: &[u8], predicate: F) -> Option<String>
where
    F: Fn(u8) -> bool,
{
    let mut reg_name = String::new();

    let mut i = 0;
    while i < src.len() {
        let byte = *unsafe { src.get_unchecked(i) };
        match byte {
                b'%' => {
                    reg_name.push('%');
                    for _ in 0..2 {
                        i += 1;
                        let digit = src
                            .get(i)
                            .filter(|&&b| is_hex_dig(b))?;
                        reg_name.push(*digit as char);
                    }
                    i += 1;
                }
                b'!'        |
                b'$'..=b',' | // '$', '&', ''', '(', ')', '*', '+', ','
                b';'        |
                b'=' => {
                    reg_name.push(byte as char);
                    i += 1;
                }
                b if is_unreserved(b) || predicate(b) => {
                    reg_name.push(b as char);
                    i += 1;
                }
                _ => break,
            }
    }
    Some(reg_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_non_hex_dig_should_be_none() {
        assert!(parse_hex_dig(b'@').is_none());
        assert!(parse_hex_dig(b'K').is_none());
        // hex dig is only capital
        assert!(parse_hex_dig(b'b').is_none());
        // 0x prefix is not accepted
        assert!(parse_hex_dig(b'x').is_none());
    }

    #[test]
    fn valid_hex_dig_values() {
        for digit in 0..10 {
            assert_eq!(Some(digit), parse_hex_dig(b'0' + digit));
        }
        for letter in 0..6 {
            assert_eq!(Some(letter + 10), parse_hex_dig(b'A' + letter));
        }
    }
}
