//! PNG encoder

use super::filter::{filter_scanline, choose_best_filter};
use super::chunks::ChunkWriter;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

pub struct PngEncoder {
    width: u32,
    height: u32,
    compression_level: u32,
}

impl PngEncoder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            compression_level: 6, // Default compression level (0-9)
        }
    }

    /// Set compression level (0-9, where 9 is maximum compression)
    pub fn with_compression(mut self, level: u32) -> Self {
        self.compression_level = level.min(9);
        self
    }

    /// Encode RGB pixels to PNG
    pub fn encode_rgb(&self, rgb_pixels: &[u8]) -> Result<Vec<u8>, String> {
        let expected_size = (self.width * self.height * 3) as usize;
        if rgb_pixels.len() != expected_size {
            return Err(format!(
                "Invalid pixel data size: expected {} bytes, got {}",
                expected_size,
                rgb_pixels.len()
            ));
        }

        println!("Encoding PNG (lossless compression)...");
        println!("  Image size: {}x{}", self.width, self.height);
        println!("  Compression level: {}", self.compression_level);

        // Step 1: Apply filters to each scanline
        let filtered_data = self.filter_image(rgb_pixels, 3)?;
        println!("  Filtered {} scanlines", self.height);

        // Step 2: Compress with DEFLATE
        let compressed_data = self.compress_data(&filtered_data)?;
        println!("  Compressed: {} → {} bytes", filtered_data.len(), compressed_data.len());

        // Step 3: Write PNG chunks
        let png_data = self.write_png_file(&compressed_data)?;
        println!("  Final PNG size: {} bytes", png_data.len());

        Ok(png_data)
    }

    /// Encode RGBA pixels to PNG
    pub fn encode_rgba(&self, rgba_pixels: &[u8]) -> Result<Vec<u8>, String> {
        let expected_size = (self.width * self.height * 4) as usize;
        if rgba_pixels.len() != expected_size {
            return Err(format!(
                "Invalid pixel data size: expected {} bytes, got {}",
                expected_size,
                rgba_pixels.len()
            ));
        }

        println!("Encoding PNG with alpha channel...");
        let filtered_data = self.filter_image(rgba_pixels, 4)?;
        let compressed_data = self.compress_data(&filtered_data)?;
        let png_data = self.write_png_file_rgba(&compressed_data)?;

        Ok(png_data)
    }

    /// Apply filters to image data
    fn filter_image(&self, pixels: &[u8], bytes_per_pixel: usize) -> Result<Vec<u8>, String> {
        let row_size = (self.width as usize) * bytes_per_pixel;
        let mut filtered = Vec::new();
        let mut prev_scanline: Option<Vec<u8>> = None;

        for y in 0..self.height as usize {
            let start = y * row_size;
            let end = start + row_size;
            let scanline = &pixels[start..end];

            // Choose best filter for this scanline
            let filter_type = choose_best_filter(
                scanline,
                prev_scanline.as_deref(),
                bytes_per_pixel,
            );

            // Apply filter
            let filtered_scanline = filter_scanline(
                scanline,
                prev_scanline.as_deref(),
                filter_type,
                bytes_per_pixel,
            );

            filtered.extend_from_slice(&filtered_scanline);
            prev_scanline = Some(scanline.to_vec());
        }

        Ok(filtered)
    }

    /// Compress data using DEFLATE (zlib)
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let compression = match self.compression_level {
            0 => Compression::none(),
            1 => Compression::fast(),
            9 => Compression::best(),
            n => Compression::new(n),
        };

        let mut encoder = ZlibEncoder::new(Vec::new(), compression);
        encoder.write_all(data).map_err(|e| format!("Compression failed: {}", e))?;
        encoder.finish().map_err(|e| format!("Compression finalization failed: {}", e))
    }

    /// Write PNG file structure (RGB)
    fn write_png_file(&self, compressed_data: &[u8]) -> Result<Vec<u8>, String> {
        let mut writer = ChunkWriter::new();
        
        writer.write_signature();
        writer.write_ihdr(self.width, self.height, 2); // Color type 2 = RGB
        writer.write_idat(compressed_data);
        writer.write_iend();
        
        Ok(writer.finish())
    }

    /// Write PNG file structure (RGBA)
    fn write_png_file_rgba(&self, compressed_data: &[u8]) -> Result<Vec<u8>, String> {
        let mut writer = ChunkWriter::new();
        
        writer.write_signature();
        writer.write_ihdr(self.width, self.height, 6); // Color type 6 = RGBA
        writer.write_idat(compressed_data);
        writer.write_iend();
        
        Ok(writer.finish())
    }
}
