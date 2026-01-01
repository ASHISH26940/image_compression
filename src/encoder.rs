//! Complete JPEG encoder pipeline

use crate::color::rgb_image_to_ycbcr;
use crate::transform::{extract_block, level_shift, forward_dct};
use crate::quantization::{quantize, scale_quantization_table, STANDARD_LUMA_QTABLE, STANDARD_CHROMA_QTABLE};
use crate::entropy::zigzag::zig_zag;
use crate::entropy::rle::{run_length_encode_ac, encode_dc_diff};
use crate::entropy::huffman::{encode_dc, encode_ac, BitWriter};
use crate::format::JpegWriter;

pub struct JpegEncoder {
    width: u16,
    height: u16,
    quality: u8,
    color: bool, // New: whether to encode in color
}

impl JpegEncoder {
    pub fn new(width: u16, height: u16, quality: u8) -> Self {
        let quality = quality.clamp(1, 100);
        Self { width, height, quality, color: true } // Default to color
    }

    /// Set grayscale mode
    pub fn grayscale(mut self) -> Self {
        self.color = false;
        self
    }

    /// Encode RGB image to JPEG (color or grayscale)
    pub fn encode(&self, rgb_pixels: &[u8]) -> Result<Vec<u8>, String> {
        let expected_size = self.width as usize * self.height as usize * 3;
        if rgb_pixels.len() != expected_size {
            return Err(format!(
                "Invalid pixel data size: expected {} bytes ({}x{}x3), got {} bytes",
                expected_size, self.width, self.height, rgb_pixels.len()
            ));
        }

        println!("Converting RGB to YCbCr...");
        let (y_channel, cb_channel, cr_channel) = 
            rgb_image_to_ycbcr(rgb_pixels, self.width as usize, self.height as usize);

        if self.color {
            self.encode_color(&y_channel, &cb_channel, &cr_channel)
        } else {
            self.encode_grayscale(&y_channel)
        }
    }

    /// Encode as grayscale (Y channel only)
    fn encode_grayscale(&self, y_channel: &[f32]) -> Result<Vec<u8>, String> {
        println!("Encoding grayscale JPEG (Y channel only)...");
        let y_qtable = scale_quantization_table(&STANDARD_LUMA_QTABLE, self.quality);
        let scan_data = self.encode_channel(y_channel, &y_qtable, false)?;

        println!("Writing JPEG file... ({} bytes of scan data)", scan_data.len());
        let mut writer = JpegWriter::new();
        writer.write_headers_grayscale(self.width, self.height);
        writer.write_scan_data(&scan_data);
        writer.write_eoi();

        Ok(writer.finish())
    }

    /// Encode as COLOR (Y, Cb, Cr channels)
    fn encode_color(&self, y_channel: &[f32], cb_channel: &[f32], cr_channel: &[f32]) 
        -> Result<Vec<u8>, String> 
    {
        println!("Encoding color JPEG (Y, Cb, Cr channels)...");
        
        let y_qtable = scale_quantization_table(&STANDARD_LUMA_QTABLE, self.quality);
        let c_qtable = scale_quantization_table(&STANDARD_CHROMA_QTABLE, self.quality);

        // Encode all three channels with interleaved MCUs
        let scan_data = self.encode_interleaved(
            y_channel, cb_channel, cr_channel,
            &y_qtable, &c_qtable
        )?;

        println!("Writing JPEG file... ({} bytes of scan data)", scan_data.len());
        let mut writer = JpegWriter::new();
        writer.write_headers_color(self.width, self.height);
        writer.write_scan_data(&scan_data);
        writer.write_eoi();

        Ok(writer.finish())
    }

    /// Encode three channels with interleaved MCUs
    fn encode_interleaved(
        &self,
        y_channel: &[f32],
        cb_channel: &[f32],
        cr_channel: &[f32],
        y_qtable: &[[f32; 8]; 8],
        c_qtable: &[[f32; 8]; 8],
    ) -> Result<Vec<u8>, String> {
        let mut bit_writer = BitWriter::new();
        
        let mut y_prev_dc = 0i32;
        let mut cb_prev_dc = 0i32;
        let mut cr_prev_dc = 0i32;

        let width = self.width as usize;
        let height = self.height as usize;
        let blocks_h = (width + 7) / 8;
        let blocks_v = (height + 7) / 8;
        
        let total_blocks = blocks_h * blocks_v;
        println!("  Processing {} MCUs ({} x {})...", total_blocks, blocks_h, blocks_v);

        for block_y in 0..blocks_v {
            for block_x in 0..blocks_h {
                // Encode Y block
                let y_block = self.encode_single_block(
                    y_channel, width, block_x, block_y,
                    y_qtable, &mut y_prev_dc, &mut bit_writer, false
                );
                
                // Encode Cb block
                let cb_block = self.encode_single_block(
                    cb_channel, width, block_x, block_y,
                    c_qtable, &mut cb_prev_dc, &mut bit_writer, true
                );
                
                // Encode Cr block
                let cr_block = self.encode_single_block(
                    cr_channel, width, block_x, block_y,
                    c_qtable, &mut cr_prev_dc, &mut bit_writer, true
                );
            }
        }

        Ok(bit_writer.finish())
    }

    /// Encode a single 8x8 block
    fn encode_single_block(
        &self,
        channel: &[f32],
        width: usize,
        block_x: usize,
        block_y: usize,
        qtable: &[[f32; 8]; 8],
        prev_dc: &mut i32,
        bit_writer: &mut BitWriter,
        is_chroma: bool,
    ) {
        let mut block = extract_block(channel, width, block_x * 8, block_y * 8);
        level_shift(&mut block);
        let dct_block = forward_dct(&block);
        let quantized = quantize(&dct_block, qtable);
        let zigzag = zig_zag(&quantized);

        let dc_diff = encode_dc_diff(zigzag[0], *prev_dc);
        encode_dc(bit_writer, dc_diff, is_chroma);
        *prev_dc = zigzag[0];

        let ac_symbols = run_length_encode_ac(&zigzag);
        encode_ac(bit_writer, &ac_symbols);
    }

    /// Encode a single channel (for grayscale or separate encoding)
    fn encode_channel(&self, channel: &[f32], qtable: &[[f32; 8]; 8], is_chroma: bool) 
        -> Result<Vec<u8>, String> 
    {
        let mut bit_writer = BitWriter::new();
        let mut prev_dc = 0i32;

        let width = self.width as usize;
        let height = self.height as usize;
        let blocks_h = (width + 7) / 8;
        let blocks_v = (height + 7) / 8;
        
        let total_blocks = blocks_h * blocks_v;
        println!("  Processing {} blocks ({} x {})...", total_blocks, blocks_h, blocks_v);

        for block_y in 0..blocks_v {
            for block_x in 0..blocks_h {
                self.encode_single_block(
                    channel, width, block_x, block_y,
                    qtable, &mut prev_dc, &mut bit_writer, is_chroma
                );
            }
        }

        Ok(bit_writer.finish())
    }
}
