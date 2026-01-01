//! JPEG file writer

use super::markers::Marker;
use crate::quantization::{STANDARD_LUMA_QTABLE, STANDARD_CHROMA_QTABLE};
use crate::entropy::zigzag::ZIGZAG_ORDER;

pub struct JpegWriter {
    buffer: Vec<u8>,
}

impl JpegWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(65536),
        }
    }

    fn write_marker(&mut self, marker: Marker) {
        self.buffer.extend_from_slice(&marker.to_bytes());
    }

    fn write_u16(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn write_soi(&mut self) {
        self.write_marker(Marker::SOI);
    }

    pub fn write_app0(&mut self) {
        self.write_marker(Marker::APP0);
        self.write_u16(16);
        self.write_bytes(b"JFIF\0");
        self.write_u8(1);
        self.write_u8(1);
        self.write_u8(0);
        self.write_u16(1);
        self.write_u16(1);
        self.write_u8(0);
        self.write_u8(0);
    }

    pub fn write_dqt(&mut self, table_id: u8, table: &[[u8; 8]; 8]) {
        self.write_marker(Marker::DQT);
        self.write_u16(67);
        self.write_u8(table_id);
        
        for &(row, col) in ZIGZAG_ORDER.iter() {
            self.write_u8(table[row][col]);
        }
    }

    pub fn write_sof0_grayscale(&mut self, width: u16, height: u16) {
        self.write_marker(Marker::SOF0);
        self.write_u16(11);
        self.write_u8(8);
        self.write_u16(height);
        self.write_u16(width);
        self.write_u8(1);
        
        self.write_u8(1);
        self.write_u8(0x11);
        self.write_u8(0);
    }

    /// SOF0 for COLOR (3 components: Y, Cb, Cr)
    pub fn write_sof0_color(&mut self, width: u16, height: u16) {
        self.write_marker(Marker::SOF0);
        self.write_u16(17); // Length for 3 components
        self.write_u8(8);   // Precision
        self.write_u16(height);
        self.write_u16(width);
        self.write_u8(3);   // 3 components
        
        // Y component
        self.write_u8(1);    // Component ID
        self.write_u8(0x11); // Sampling (1x1, no subsampling)
        self.write_u8(0);    // Quantization table 0
        
        // Cb component
        self.write_u8(2);    // Component ID
        self.write_u8(0x11); // Sampling (1x1)
        self.write_u8(1);    // Quantization table 1
        
        // Cr component
        self.write_u8(3);    // Component ID
        self.write_u8(0x11); // Sampling (1x1)
        self.write_u8(1);    // Quantization table 1
    }

    pub fn write_dht_standard(&mut self) {
        // DC Luminance (Table K.3)
        self.write_marker(Marker::DHT);
        let dc_luma_bits: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
        let dc_luma_vals: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        self.write_u16(19 + 12);
        self.write_u8(0x00); // DC table 0
        self.write_bytes(&dc_luma_bits);
        self.write_bytes(&dc_luma_vals);

        // AC Luminance (Table K.5)
        self.write_marker(Marker::DHT);
        let ac_luma_bits: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
        let ac_luma_vals: [u8; 162] = [
            0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12,
            0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
            0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08,
            0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
            0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16,
            0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
            0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
            0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
            0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
            0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
            0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
            0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98,
            0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
            0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6,
            0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
            0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4,
            0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
            0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea,
            0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
            0xf9, 0xfa,
        ];
        self.write_u16(19 + 162);
        self.write_u8(0x10); // AC table 0
        self.write_bytes(&ac_luma_bits);
        self.write_bytes(&ac_luma_vals);

        // DC Chrominance (same as luma for simplicity)
        self.write_marker(Marker::DHT);
        self.write_u16(19 + 12);
        self.write_u8(0x01); // DC table 1
        self.write_bytes(&dc_luma_bits);
        self.write_bytes(&dc_luma_vals);

        // AC Chrominance (same as luma for simplicity)
        self.write_marker(Marker::DHT);
        self.write_u16(19 + 162);
        self.write_u8(0x11); // AC table 1
        self.write_bytes(&ac_luma_bits);
        self.write_bytes(&ac_luma_vals);
    }

    pub fn write_sos_grayscale(&mut self) {
        self.write_marker(Marker::SOS);
        self.write_u16(8);
        self.write_u8(1);
        
        self.write_u8(1);
        self.write_u8(0x00);
        
        self.write_u8(0);
        self.write_u8(63);
        self.write_u8(0);
    }

    /// SOS for COLOR (3 components)
    pub fn write_sos_color(&mut self) {
        self.write_marker(Marker::SOS);
        self.write_u16(12); // Length
        self.write_u8(3);   // 3 components
        
        // Y component
        self.write_u8(1);    // Component ID
        self.write_u8(0x00); // DC table 0, AC table 0
        
        // Cb component
        self.write_u8(2);    // Component ID
        self.write_u8(0x11); // DC table 1, AC table 1
        
        // Cr component
        self.write_u8(3);    // Component ID
        self.write_u8(0x11); // DC table 1, AC table 1
        
        self.write_u8(0);   // Start of spectral selection
        self.write_u8(63);  // End of spectral selection
        self.write_u8(0);   // Successive approximation
    }

    pub fn write_scan_data(&mut self, data: &[u8]) {
        self.write_bytes(data);
    }

    pub fn write_eoi(&mut self) {
        self.write_marker(Marker::EOI);
    }

    pub fn write_headers_grayscale(&mut self, width: u16, height: u16) {
        self.write_soi();
        self.write_app0();
        self.write_dqt(0, &STANDARD_LUMA_QTABLE);
        self.write_sof0_grayscale(width, height);
        self.write_dht_standard();
        self.write_sos_grayscale();
    }

    /// Write headers for COLOR JPEG
    pub fn write_headers_color(&mut self, width: u16, height: u16) {
        self.write_soi();
        self.write_app0();
        self.write_dqt(0, &STANDARD_LUMA_QTABLE);
        self.write_dqt(1, &STANDARD_CHROMA_QTABLE);
        self.write_sof0_color(width, height);
        self.write_dht_standard();
        self.write_sos_color();
    }

    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}
