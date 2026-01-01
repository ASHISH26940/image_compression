//! PNG chunk writing

use std::io::Write;
use crc32fast::Hasher;

pub struct ChunkWriter {
    buffer: Vec<u8>,
}

impl ChunkWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Write PNG signature
    pub fn write_signature(&mut self) {
        // PNG signature: 137 80 78 71 13 10 26 10
        self.buffer.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    /// Write a PNG chunk
    pub fn write_chunk(&mut self, chunk_type: &[u8; 4], data: &[u8]) {
        // Length (4 bytes, big-endian)
        self.buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
        
        // Chunk type (4 bytes)
        self.buffer.extend_from_slice(chunk_type);
        
        // Chunk data
        self.buffer.extend_from_slice(data);
        
        // CRC32 of type + data
        let mut hasher = Hasher::new();
        hasher.update(chunk_type);
        hasher.update(data);
        let crc = hasher.finalize();
        self.buffer.extend_from_slice(&crc.to_be_bytes());
    }

    /// Write IHDR chunk (Image Header)
    pub fn write_ihdr(&mut self, width: u32, height: u32, color_type: u8) {
        let mut data = Vec::new();
        data.extend_from_slice(&width.to_be_bytes());      // Width
        data.extend_from_slice(&height.to_be_bytes());     // Height
        data.push(8);                                      // Bit depth (8 bits)
        data.push(color_type);                             // Color type (2=RGB, 6=RGBA)
        data.push(0);                                      // Compression method (0=DEFLATE)
        data.push(0);                                      // Filter method (0=adaptive)
        data.push(0);                                      // Interlace method (0=none)
        
        self.write_chunk(b"IHDR", &data);
    }

    /// Write IDAT chunk (Image Data)
    pub fn write_idat(&mut self, compressed_data: &[u8]) {
        self.write_chunk(b"IDAT", compressed_data);
    }

    /// Write IEND chunk (Image End)
    pub fn write_iend(&mut self) {
        self.write_chunk(b"IEND", &[]);
    }

    /// Get the final PNG file bytes
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_signature() {
        let mut writer = ChunkWriter::new();
        writer.write_signature();
        assert_eq!(&writer.buffer[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_chunk_structure() {
        let mut writer = ChunkWriter::new();
        writer.write_chunk(b"TEST", b"data");
        
        // Should have: length(4) + type(4) + data(4) + crc(4) = 16 bytes
        assert_eq!(writer.buffer.len(), 16);
    }
}
