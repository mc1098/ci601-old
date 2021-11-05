/// Module that contains functions relating to parsing or validating of ABNF syntax types.

/// Checks if the value is a `unreserved` ABNF as defined in
/// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986)
///
/// ```text
/// unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
///
/// ALPHA = %x41-5A / %x61-7A   ; A-Z / a-z
/// DIGIT = %x30-39             ; 0-9
/// ```
///
/// ALPHA & DIGIT are checked using [`u8::is_ascii_alphanumeric`].
#[inline]
pub(crate) const fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Checks if the value is a `HEXDIG` ABNF as defined in
/// [RFC5234](https://datatracker.ietf.org/doc/html/rfc5234)
///
/// Note that HEXDIG is restricted to only allowing uppercase letters and so
/// [`u8::is_ascii_hexdigit`] cannot be used.
///
/// ```text
/// HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
///
/// DIGIT = %x30-39 ; 0-9
/// ```
#[inline]
pub(crate) const fn is_hex_dig(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'A'..=b'F')
}

/// Checks if the value is a `sub-delims` ABNF as defined in
/// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986)
///
/// ```text
/// sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
/// ```
#[inline]
pub(crate) const fn is_sub_delims(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    )
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
pub(crate) fn parse_reg_name(src: &[u8]) -> Option<String> {
    // SAFETY:
    // unreserved and sub-delims is valid ascii characters so this upholds the
    // safety requirements of parse_pct_encoded_ext
    unsafe { parse_pct_encoded_ext(src, |b| is_unreserved(b) || is_sub_delims(b)) }
}

/// Parse a sequence of bytes upto when the predicate fails to a String.
///
/// # Safety
///
/// This function assumes that all bytes parsed are valid ascii characters and
/// so the predicate given should only return true for valid ascii characters.
pub(crate) unsafe fn parse_seq<P>(src: &[u8], predicate: P) -> Option<String>
where
    P: Fn(u8) -> bool,
{
    let count = src.iter().take_while(|b| predicate(**b)).count();
    src.get(..count).map(|bytes| {
        // SAFETY:
        unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
    })
}

/// Parse a sequence of bytes as a `pct-encoded` values or other values
/// that satisfy the predicate given to a String.
///
/// Returns None if the `pct-encoded` value is invalid.
///
/// # Safety
///
/// This function assumes that all bytes parsed are valid ascii characters and
/// so the predicate given should only return true for valid ascii characters.
pub(crate) unsafe fn parse_pct_encoded_ext<P>(src: &[u8], predicate: P) -> Option<String>
where
    P: Fn(u8) -> bool,
{
    // We will use pct as a mutable reference in the take_while
    // iterator to persist the value. This will keep track of
    // if we are trying to parse a pct-encoded value.
    // pct will be 0 initially which means not to try and parse
    // HEXDIGs.
    // pct value will indicate which number HEXDIG we are parsing,
    // however because we only want to parse two we only care about
    // pct values below 4, 0b1 is HEXDIG 1 and 0b10 is HEXDIG 2, so
    // the bit AND reverts pct back to zero on the second parse of
    // HEXDIG.
    let mut pct = 0;
    let count = src
        .iter()
        .take_while(|b| {
            if pct != 0 {
                pct = (pct << 1) & 0b11;
                return is_hex_dig(**b);
            }

            if **b == b'%' {
                pct = 1;
                true
            } else {
                predicate(**b)
            }
        })
        .count();

    // check that a pct-encoded value was not part way complete when
    // iterator stopped
    if pct != 0 {
        return None;
    }

    src.get(0..count).map(|bytes| {
        // SAFETY:
        // The bytes slice has been checked for valid ascii characters
        // and ascii is always valid UTF-8 so this is safe.
        unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
    })
}

/// Parse multiple `pchar` from a sequence of bytes to a String.
///
/// Returns None if the `pct-encoded` value is invalid.
pub(crate) fn parse_pchar(src: &[u8]) -> Option<String> {
    // SAFETY
    // pchar are valid ascii characters and the predicate provided
    // does not allow any other characters so this satisfies the
    // safety requirement of parse_pchar_ext.
    unsafe { parse_pchar_ext(src, |_| false) }
}

/// Parse a `pchar` and other characters allowed by the predicate given from a sequence of
/// bytes to a String.
///
/// Returns None if the `pct-encoded` value is invalid.
///
/// # Safety
/// This function assumes that all bytes allowed will be a valid ascii character,
/// therefore the predicate must only allow valid ascii characters.
pub(crate) unsafe fn parse_pchar_ext<P>(src: &[u8], predicate: P) -> Option<String>
where
    P: Fn(u8) -> bool,
{
    // SAFETY:
    // pchar are valid ascii characters and we assume that anything
    // that matches the predicate is also a valid ascii character
    parse_pct_encoded_ext(src, |b| {
        is_unreserved(b) || is_sub_delims(b) || matches!(b, b':' | b'@') || predicate(b)
    })
}

/// Parse a fragment or query from a sequence of bytes as a String.
///
/// The ABNF types fragment and query share the same syntax so both
/// can use this function.
///
/// This will return None if a pct-encoded sequence is invalid.
///
/// ```text
/// fragment / query = *( pchar / "/" / "?" )
///
/// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
pub(crate) fn parse_frag_or_query(src: &[u8]) -> Option<String> {
    // SAFETY:
    // pchar and "/" and "?" are valid ascii characters so this
    // satisfies the safety requirements of parse_pct_encoded_ext
    unsafe { parse_pchar_ext(src, |b| matches!(b, b'/' | b'?')) }
}

/// Parse a single `HEXDIG` into a [`u8`] value.
///
/// This function will most likely be used to parse multiple `HEXDIG`s so it is important to
/// shift the accumulating value up by 4 bits before adding the new parsed `HEXDIG`.
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

#[cfg(test)]
mod tests {
    use super::{parse_hex_dig, parse_reg_name};

    #[test]
    fn reg_name_with_single_percent_is_not_valid() {
        assert!(parse_reg_name(b"%").is_none())
    }

    #[test]
    fn reg_name_pct_one_hex_dig_is_not_valid() {
        assert!(parse_reg_name(b"%1").is_none());
    }

    #[test]
    fn reg_name_pct_encoded_values() {
        assert_eq!(Some("%1A".to_owned()), parse_reg_name(b"%1A"));
        assert_eq!(Some("%1A%F6".to_owned()), parse_reg_name(b"%1A%F6"));
        assert_eq!(Some("%FF%FF".to_owned()), parse_reg_name(b"%FF%FF"));
    }

    #[test]
    fn parsing_reg_name_stops_at_invalid_char() {
        assert_eq!(Some("hello".to_owned()), parse_reg_name(b"hello:there"));
    }

    #[test]
    fn valid_reg_name_examples() {
        assert_eq!(
            Some("h12*$~;%FF33%01".to_owned()),
            parse_reg_name(b"h12*$~;%FF33%01@ignore_this_part")
        );
    }

    #[test]
    fn non_hex_dig_is_none_on_parse() {
        // symbol
        assert!(parse_hex_dig(b'@').is_none());
        // letter greater than 'F'
        assert!(parse_hex_dig(b'G').is_none());
        // not capital letter is not acceptable
        assert!(parse_hex_dig(b'a').is_none());
    }

    #[test]
    fn all_valid_hex_digs() {
        for (i, digit) in (b'0'..=b'9').into_iter().enumerate() {
            assert_eq!(Some(i as u8), parse_hex_dig(digit));
        }

        for (i, letter) in (b'A'..=b'F')
            .into_iter()
            .enumerate()
            .map(|(i, l)| (i + 10, l))
        {
            assert_eq!(Some(i as u8), parse_hex_dig(letter));
        }
    }
}
