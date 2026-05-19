//! Bounds checking utilities for BLP parsers

use super::error::Error;
use super::types::ParseResult;
use log::error;

/// Check if a given offset and size are within the bounds of input data
///
/// This helper function consolidates the bounds checking logic that was duplicated
/// across multiple BLP parsing functions.
pub fn check_bounds(input: &[u8], offset: u32, size: u32, mipmap_index: usize) -> ParseResult<()> {
    // Allow the special case where offset == len and size == 0. This represents
    // an empty mipmap located exactly at EOF and should be considered valid.
    let offset_usize = offset as usize;
    let size_usize = size as usize;
    let len = input.len();

    // If offset is strictly greater than length it's always out of bounds. If
    // offset equals length then it's only valid when size is zero.
    if offset_usize > len || (offset_usize == len && size_usize > 0) {
        error!(
            "Offset of mipmap {} is out of bounds! {} >= {}",
            mipmap_index,
            offset,
            len
        );
        return Err(Error::OutOfBounds {
            offset: offset_usize,
            size: 0,
        });
    }

    // Check if offset + size extends beyond input bounds. Use checked_add to
    // avoid integer overflow on the addition.
    if offset_usize.checked_add(size_usize).unwrap_or(len + 1) > len {
        error!(
            "Offset+size of mipmap {} is out of bounds! {} > {}",
            mipmap_index,
            offset + size,
            len
        );
        return Err(Error::OutOfBounds {
            offset: offset_usize,
            size: size_usize,
        });
    }

    Ok(())
}

/// Get a slice from input data after bounds checking
///
/// Convenience function that checks bounds and returns the slice if valid.
pub fn get_bounded_slice(
    input: &[u8],
    offset: u32,
    size: u32,
    mipmap_index: usize,
) -> ParseResult<&[u8]> {
    check_bounds(input, offset, size, mipmap_index)?;
    Ok(&input[offset as usize..(offset + size) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        assert!(check_bounds(&data, 0, 3, 0).is_ok());
        assert!(check_bounds(&data, 2, 3, 0).is_ok());
        assert!(check_bounds(&data, 0, 5, 0).is_ok());
    }

    #[test]
    fn test_offset_out_of_bounds() {
        let data = vec![1, 2, 3];
        assert!(check_bounds(&data, 5, 1, 0).is_err());
        // offset == len is allowed only when size == 0 (empty slice at EOF)
        assert!(check_bounds(&data, 3, 0, 0).is_ok());
        assert!(check_bounds(&data, 3, 1, 0).is_err());
    }

    #[test]
    fn test_size_out_of_bounds() {
        let data = vec![1, 2, 3];
        assert!(check_bounds(&data, 1, 3, 0).is_err()); // 1 + 3 > 3
        assert!(check_bounds(&data, 0, 4, 0).is_err()); // 0 + 4 > 3
    }

    #[test]
    fn test_get_bounded_slice() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = get_bounded_slice(&data, 1, 3, 0).unwrap();
        assert_eq!(slice, &[2, 3, 4]);
    }
}
