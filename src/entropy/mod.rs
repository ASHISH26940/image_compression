//! Entropy encoding (zigzag, RLE, Huffman)

pub mod zigzag;
pub mod rle;
pub mod huffman;

pub use zigzag::{zig_zag, ZIGZAG_ORDER};
pub use rle::{run_length_encode_ac, encode_dc_diff, RLESymbol};
pub use huffman::{encode_dc, encode_ac, BitWriter};
