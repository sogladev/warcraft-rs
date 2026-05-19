use super::super::bounds::get_bounded_slice;
use super::super::reader::{ByteReader, Cursor};
use super::super::types::ParseResult;
use crate::types::*;
use log::*;

pub fn parse_raw1<'a>(
    blp_header: &BlpHeader,
    original_input: &'a [u8],
    offsets: &[u32; 16],
    sizes: &[u32; 16],
    images: &mut Vec<Raw1Image>,
    _input: &'a [u8],
) -> ParseResult<()> {
    let mut read_image = |i: usize| -> ParseResult<()> {
        let offset = offsets[i];
        let size = sizes[i];
        let image_bytes = get_bounded_slice(original_input, offset, size, i)?;
        let mut reader = Cursor::new(image_bytes);

        let n = blp_header.mipmap_pixels(i);
        let indexed_rgb = reader.read_bytes(n as usize)?;

        let an = (n * blp_header.alpha_bits()).div_ceil(8);
        let indexed_alpha = reader.read_bytes(an as usize)?;

        images.push(Raw1Image {
            indexed_rgb,
            indexed_alpha,
        });
        Ok(())
    };

    read_image(0)?;
    if blp_header.has_mipmaps() {
        for (i, &size) in sizes.iter().enumerate().skip(1) {
            if i > blp_header.mipmaps_count() {
                break;
            }
            let offset = offsets[i];
            if size == 0 || offset == 0 {
                trace!("Size/offset indicates no data for mipmap {i}; stopping");
                break;
            }
            if (offset as usize) >= original_input.len() {
                trace!("Offset of mipmap {i} is at/after EOF: {} >= {}", offset, original_input.len());
                break;
            }
            if (offset as usize).checked_add(size as usize).unwrap_or(usize::MAX) > original_input.len() {
                trace!("Size of mipmap {i} exceeds file bounds; stopping");
                break;
            }

            read_image(i)?;
        }
    }
    Ok(())
}
