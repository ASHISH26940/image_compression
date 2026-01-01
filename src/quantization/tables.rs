/// Standard JPEG luminance quantization table (quality 50)
/// Values are in zigzag order for the 8x8 DCT block
pub const STANDARD_LUMA_QTABLE: [[u8; 8]; 8] = [
    [16, 11, 10, 16,  24,  40,  51,  61],
    [12, 12, 14, 19,  26,  58,  60,  55],
    [14, 13, 16, 24,  40,  57,  69,  56],
    [14, 17, 22, 29,  51,  87,  80,  62],
    [18, 22, 37, 56,  68, 109, 103,  77],
    [24, 35, 55, 64,  81, 104, 113,  92],
    [49, 64, 78, 87, 103, 121, 120, 101],
    [72, 92, 95, 98, 112, 100, 103,  99],
];

/// Standard JPEG chrominance quantization table (quality 50)
pub const STANDARD_CHROMA_QTABLE: [[u8; 8]; 8] = [
    [17, 18, 24, 47, 99, 99, 99, 99],
    [18, 21, 26, 66, 99, 99, 99, 99],
    [24, 26, 56, 99, 99, 99, 99, 99],
    [47, 66, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
];

pub fn scale_quantization_table(base_table:&[[u8;8];8],quality:u8)->[[f32;8];8]
{
    let quality=quality.clamp(1, 100);

    let scale_factor= if quality<50{
        5000.0/quality as f32
    }else{
        200.0 - quality as f32 *2.0
    };

    let mut scaled=[[0.0f32;8];8];
    for i in 0..8{
        for j in 0..8 {
            let val=((base_table[i][j] as f32 * scale_factor+50.0)/100.0).floor();

            scaled[i][j]=val.max(1.0);
        }
    }
    scaled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_quality_100() {
        let scaled = scale_quantization_table(&STANDARD_LUMA_QTABLE, 100);
        // Quality 100 should result in all 1s (minimum quantization)
        assert!((scaled[0][0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_scale_quality_50() {
        let scaled = scale_quantization_table(&STANDARD_LUMA_QTABLE, 50);
        // Quality 50 should match the base table
        assert!((scaled[0][0] - 16.0).abs() < 1.0);
    }
}