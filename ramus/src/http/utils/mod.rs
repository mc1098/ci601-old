pub(crate) mod abnf;

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
