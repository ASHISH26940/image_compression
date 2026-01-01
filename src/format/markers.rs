//! JPEG marker definitions

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Marker {
    // Start and End
    SOI = 0xD8,   // Start of Image
    EOI = 0xD9,   // End of Image
    
    // JFIF
    APP0 = 0xE0,  // Application segment 0
    
    // Frame types
    SOF0 = 0xC0,  // Start of Frame (Baseline DCT)
    
    // Huffman and Quantization
    DHT = 0xC4,   // Define Huffman Table
    DQT = 0xDB,   // Define Quantization Table
    
    // Scan
    SOS = 0xDA,   // Start of Scan
}

impl Marker {
    pub fn to_bytes(self) -> [u8; 2] {
        [0xFF, self as u8]
    }
}
