/// Crate Module:
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
    let bytes = src.iter().take_while(|b| predicate(**b)).copied().collect();

    // SAFETY:
    // assumes that predicate given only allows valid ascii characters which are valid UTF-8
    // and so satisfies the safety requirements of from_utf8_unchecked.
    // Note: unsafe block used locally to denote unsafe part of function
    #[allow(unused_unsafe)]
    Some(unsafe { String::from_utf8_unchecked(bytes) })
}

/// Parse a sequence of bytes as a `pct-encoded` values or other values
/// that satisfy the predicate given to a String.
///
/// Returns None if the `pct-encoded` value is invalid.
/// A predicate that accepts the '%' octet will still cause this function to
/// return None if that '%' is not followed by two valid HEXDIG.
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
    let bytes = src
        .iter()
        .take_while(|b| {
            if pct != 0 {
                return if is_hex_dig(**b) {
                    // only change pct if b is a valid HEXDIG
                    pct = (pct << 1) & 0b11;
                    true
                } else {
                    false
                };
            }

            if **b == b'%' {
                pct = 1;
                true
            } else {
                predicate(**b)
            }
        })
        .copied()
        .collect();

    // check that a pct-encoded value was not part way complete when
    // iterator stopped
    if pct != 0 {
        return None;
    }

    // SAFETY:
    // The bytes slice has been checked for valid ascii characters
    // and ascii is always valid UTF-8 so this is safe.
    // Note: unsafe block used here to denote the part of the function
    // that is unsafe
    #[allow(unused_unsafe)]
    Some(unsafe { String::from_utf8_unchecked(bytes) })
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

macro_rules! parse_hex_uint_impl {
    ($($fn_name:ident => $uint:ty,)*) => {
        $(
            /// Parse a number of HEXDIGs into the uint type.
            ///
            /// This function will parse a number of octets upto the number
            /// of nibbles in the uint type while those octets are valid HEXDIGs.
            ///
            /// The number of HEXDIGs parsed is variable but at least one HEXDIG
            /// must exists in order to return a Some value.
            #[allow(dead_code)]
            pub(crate) fn $fn_name(src: &[u8]) -> Option<($uint, &[u8])> {
                // We use the number of nibbles for this type so we can
                // only take that many HEXDIGs - this guarantees that
                // the shift left and adds in the fold cannot overflow the
                // uint type.
                let nibbles = <$uint>::BITS as usize / 4;

                // use fold_count to count the number of iterations of the fold
                // closure - this is easier to read then doing the same as part of the
                // accumulating value.
                let mut fold_count = 0;

                src.iter()
                    .take(nibbles)
                    // map -> take_while -> map
                    // so that we only take up to a failed parse of HEXDIG
                    // and unwrap known Some values so the unwrap is guaranteed
                    // to not panic!
                    .map(|b| parse_hex_dig(*b))
                    .take_while(Option::is_some)
                    .map(Option::unwrap)
                    .fold(None, |acc, hex| {
                        fold_count += 1;
                        Some((acc.unwrap_or_default() << 4) + hex as $uint)
                    })
                    .map(|hex| (hex, src.get(fold_count..).unwrap_or_default()))
            }
        )*
    };
}

parse_hex_uint_impl! {
    parse_hex_u8 => u8,
    parse_hex_u16 => u16,
}

#[cfg(test)]
mod tests {
    use super::{
        parse_hex_dig, parse_hex_u16, parse_hex_u8, parse_pct_encoded_ext, parse_reg_name,
    };

    #[test]
    fn empty_slice_cannot_be_parsed_as_hex_u8() {
        assert!(parse_hex_u8(&[]).is_none());
    }

    #[test]
    fn invalid_hex_dig_prefix_prevents_parsing_hex() {
        assert!(parse_hex_u8(b"@1").is_none());
        // only capital ABCDEF is a valid HEXDIG
        assert!(parse_hex_u8(b"a1").is_none());
    }

    #[test]
    fn multiple_hex_digs_can_be_parsed_upto_nibble_limit_of_uint_type() {
        // u8 has 2 nibbles
        assert_eq!(Some((0xa, [].as_ref())), parse_hex_u8(b"A"));
        assert_eq!(Some((0x14, [].as_ref())), parse_hex_u8(b"14"));
        assert_eq!(Some((0xff, b"1".as_ref())), parse_hex_u8(b"FF1"));

        // u16 has 4 nibbles
        assert_eq!(Some((0xa, [].as_ref())), parse_hex_u16(b"A"));
        assert_eq!(Some((0x14, [].as_ref())), parse_hex_u16(b"14"));
        assert_eq!(Some((0xff1, [].as_ref())), parse_hex_u16(b"FF1"));
        assert_eq!(Some((0xb1f8, [].as_ref())), parse_hex_u16(b"B1F8"));
        assert_eq!(Some((0xaaff, b"A".as_ref())), parse_hex_u16(b"AAFFA"));
    }

    #[test]
    fn single_percent_is_not_a_valid_pct_encoded() {
        assert!(unsafe { parse_pct_encoded_ext(b"%", |_| false) }.is_none());
        assert!(unsafe { parse_pct_encoded_ext(b"%", |b| b == b'%') }.is_none());
    }

    #[test]
    fn single_percent_and_hex_dig_is_not_a_valid_pct_encoded() {
        assert!(unsafe { parse_pct_encoded_ext(b"%1", |_| false) }.is_none());
    }

    #[test]
    fn valid_pct_encoded_values() {
        assert_eq!(Some("%1A".to_owned()), unsafe {
            parse_pct_encoded_ext(b"%1A", |_| false)
        });
        assert_eq!(Some("%1A%F6".to_owned()), unsafe {
            parse_pct_encoded_ext(b"%1A%F6", |_| false)
        });
        assert_eq!(Some("%FF%FF".to_owned()), unsafe {
            parse_pct_encoded_ext(b"%FF%FF", |_| false)
        });
    }

    #[test]
    fn predicate_with_pct_encoded_does_not_allow_octets_within_pct_encoded_value() {
        // predicate will allow for octets before or after a valid pct-encoded value
        // but those pct-encoded values must still be valid and cannot contain octets
        // even if permitted by the predicate
        assert_eq!(None, unsafe {
            parse_pct_encoded_ext(b"%F:F", |b| b == b':')
        });
        assert!(unsafe { parse_pct_encoded_ext(b"%F:F", |b| b == b':') }.is_none())
    }

    #[test]
    fn predicate_with_pct_encoded_allow_interleaving_octets() {
        assert_eq!(Some("%B7@%54".to_owned()), unsafe {
            parse_pct_encoded_ext(b"%B7@%54", |b| b == b'@')
        });
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
