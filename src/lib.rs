//! JPEG Encoder Library
//! 
//! A from-scratch implementation of baseline JPEG compression.

pub mod color;
pub mod transform;
pub mod quantization;
pub mod entropy;
pub mod format;
pub mod encoder;
// pub mod format;  // Uncomment when implemented
// pub mod encoder; // Uncomment when implemented

// Re-export commonly used items
pub use color::{rgb_to_ycbcr, YCbCr};
pub use encoder::JpegEncoder;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
