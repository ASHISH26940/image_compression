//! Complete JPEG encoder pipeline

use crate::color::rgb_image_to_ycbcr;
use crate::transform::{extract_block, level_shift, forward_dct};
use crate::quantization::{quantize, scale_quantization_table, STANDARD_LUMA_QTABLE};
use crate::entropy::zigzag::zig_zag;
use crate::entropy::rle::{run_length_encode_ac, encode_dc_diff};
use crate::entropy::huffman::{encode_dc, encode_ac, BitWriter};
use crate::format::JpegWriter;

pub struct JpegEncoder {
    width: u16,
    height: u16,
    quality: u8,
}

impl JpegEncoder {
    pub fn new(width: u16, height: u16, quality: u8) -> Self {
        let quality = quality.clamp(1, 100);
        Self { width, height, quality }
    }

    /// Encode RGB image to GRAYSCALE JPEG (simplified, working version)
    pub fn encode(&self, rgb_pixels: &[u8]) -> Result<Vec<u8>, String> {
    let expected_size = self.width as usize * self.height as usize * 3;
    if rgb_pixels.len() != expected_size {
        return Err(format!(
            "Invalid pixel data size: expected {} bytes ({}x{}x3), got {} bytes",
            expected_size, self.width, self.height, rgb_pixels.len()
        ));
    }

    println!("Converting RGB to YCbCr...");
    let (y_channel, _cb, _cr) = 
        rgb_image_to_ycbcr(rgb_pixels, self.width as usize, self.height as usize);

    // Verify channel size
    let expected_pixels = (self.width as usize) * (self.height as usize);
    if y_channel.len() != expected_pixels {
        return Err(format!(
            "Channel size mismatch: expected {}, got {}",
            expected_pixels, y_channel.len()
        ));
    }

    println!("Encoding grayscale JPEG (Y channel only)...");
    let y_qtable = scale_quantization_table(&STANDARD_LUMA_QTABLE, self.quality);
    let scan_data = self.encode_channel(&y_channel, &y_qtable)?;

    println!("Writing JPEG file... ({} bytes of scan data)", scan_data.len());
    let mut writer = JpegWriter::new();
    writer.write_headers_grayscale(self.width, self.height);
    writer.write_scan_data(&scan_data);
    writer.write_eoi();

    Ok(writer.finish())
}


    /// Encode a single channel (Y, Cb, or Cr)
fn encode_channel(&self, channel: &[f32], qtable: &[[f32; 8]; 8]) 
    -> Result<Vec<u8>, String> 
{
    let mut bit_writer = BitWriter::new();
    let mut prev_dc = 0i32;  // ✓ Keep DC predictor across entire image

    let width = self.width as usize;
    let height = self.height as usize;

    let blocks_h = (width + 7) / 8;
    let blocks_v = (height + 7) / 8;
    
    let total_blocks = blocks_h * blocks_v;
    println!("  Processing {} blocks ({} x {})...", total_blocks, blocks_h, blocks_v);

    for block_y in 0..blocks_v {
        for block_x in 0..blocks_h {
            let mut block = extract_block(channel, width, block_x * 8, block_y * 8);
            level_shift(&mut block);
            let dct_block = forward_dct(&block);
            let quantized = quantize(&dct_block, qtable);
            let zigzag = zig_zag(&quantized);

            // Encode DC with DPCM (continuous predictor)
            let dc_diff = encode_dc_diff(zigzag[0], prev_dc);
            encode_dc(&mut bit_writer, dc_diff, false);
            prev_dc = zigzag[0];

            // Encode AC coefficients
            let ac_symbols = run_length_encode_ac(&zigzag);
            encode_ac(&mut bit_writer, &ac_symbols);
        }
    }

    Ok(bit_writer.finish())
}


}
