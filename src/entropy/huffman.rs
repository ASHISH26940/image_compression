//! Huffman encoding with codes that MATCH the DHT we write

use super::rle::{RLESymbol, get_size_category, get_amplitude_bits};

#[derive(Debug, Clone, Copy)]
pub struct HuffmanCode {
    pub code: u16,
    pub length: u8,
}

// DC codes (Table K.3) - these are correct
pub const DC_LUMA_CODES: [HuffmanCode; 12] = [
    HuffmanCode { code: 0b00, length: 2 },
    HuffmanCode { code: 0b010, length: 3 },
    HuffmanCode { code: 0b011, length: 3 },
    HuffmanCode { code: 0b100, length: 3 },
    HuffmanCode { code: 0b101, length: 3 },
    HuffmanCode { code: 0b110, length: 3 },
    HuffmanCode { code: 0b1110, length: 4 },
    HuffmanCode { code: 0b11110, length: 5 },
    HuffmanCode { code: 0b111110, length: 6 },
    HuffmanCode { code: 0b1111110, length: 7 },
    HuffmanCode { code: 0b11111110, length: 8 },
    HuffmanCode { code: 0b111111110, length: 9 },
];

// AC codes generated from standard table - MUST match DHT!
lazy_static::lazy_static! {
    static ref AC_CODE_TABLE: Vec<HuffmanCode> = generate_ac_table();
}

fn generate_ac_table() -> Vec<HuffmanCode> {
    // This is the EXACT table from ITU-T T.81 Table K.5
    // Matches what we write in writer.rs write_dht_standard()
    let values: &[u8] = &[
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
    
    let bits: &[u8] = &[0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
    
    // Generate codes from bit lengths
    let mut codes = vec![HuffmanCode { code: 0, length: 0 }; 256];
    let mut code = 0u16;
    let mut value_idx = 0;
    
    for length in 1..=16 {
        let count = bits[length - 1] as usize;
        for _ in 0..count {
            let symbol = values[value_idx] as usize;
            codes[symbol] = HuffmanCode {
                code,
                length: length as u8,
            };
            code += 1;
            value_idx += 1;
        }
        code <<= 1;
    }
    
    codes
}

pub fn get_ac_code(run_length: u8, size: u8) -> HuffmanCode {
    let symbol = ((run_length as usize) << 4) | (size as usize);
    AC_CODE_TABLE[symbol]
}

// Rest of BitWriter stays the same...
pub struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_position: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            bit_position: 0,
        }
    }

    pub fn write_bits(&mut self, bits: u16, count: u8) {
        if count == 0 || count > 16 {
            return;
        }

        let mut bits = bits;
        let mut remaining = count;

        while remaining > 0 {
            let space_in_byte = 8 - self.bit_position;
            let bits_to_write = remaining.min(space_in_byte);

            let shift = remaining - bits_to_write;
            let mask = (1u16 << bits_to_write) - 1;
            let value = ((bits >> shift) & mask) as u8;

            self.current_byte |= value << (space_in_byte - bits_to_write);
            self.bit_position += bits_to_write;

            if self.bit_position >= 8 {
                self.flush_byte();
            }

            bits &= (1u16 << shift) - 1;
            remaining -= bits_to_write;
        }
    }

    fn flush_byte(&mut self) {
        self.buffer.push(self.current_byte);
        
        if self.current_byte == 0xFF {
            self.buffer.push(0x00);
        }
        
        self.current_byte = 0;
        self.bit_position = 0;
    }

    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_position > 0 {
            self.current_byte |= (1 << (8 - self.bit_position)) - 1;
            self.flush_byte();
        }
        self.buffer
    }
}

pub fn encode_dc(writer: &mut BitWriter, diff: i32, _is_chroma: bool) {
    let size = get_size_category(diff);
    let amplitude = get_amplitude_bits(diff, size);
    
    let code = DC_LUMA_CODES[size as usize];
    writer.write_bits(code.code, code.length);
    
    if size > 0 {
        writer.write_bits(amplitude, size);
    }
}

pub fn encode_ac(writer: &mut BitWriter, symbols: &[RLESymbol]) {
    for symbol in symbols {
        let size = get_size_category(symbol.value);
        let amplitude = get_amplitude_bits(symbol.value, size);
        
        let code = get_ac_code(symbol.run_length, size);
        writer.write_bits(code.code, code.length);
        
        if size > 0 {
            writer.write_bits(amplitude, size);
        }
    }
}
