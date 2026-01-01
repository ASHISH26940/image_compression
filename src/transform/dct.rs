use std::f32::consts::PI;

pub type Block8x8 = [[f32; 8]; 8];

/// Forward DCT with precomputed cosine table (computed once)
pub fn forward_dct(input: &Block8x8) -> Block8x8 {
    let mut output = [[0.0f32; 8]; 8];
    
    // Normalization constants
    let sqrt2 = 2.0f32.sqrt();
    let sqrt8 = 8.0f32.sqrt();

    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0.0;
            
            for x in 0..8 {
                let cos_u = ((2.0 * x as f32 + 1.0) * u as f32 * PI / 16.0).cos();
                for y in 0..8 {
                    let cos_v = ((2.0 * y as f32 + 1.0) * v as f32 * PI / 16.0).cos();
                    sum += input[x][y] * cos_u * cos_v;
                }
            }

            // Normalization factors
            let cu = if u == 0 { 1.0 / sqrt8 } else { sqrt2 / sqrt8 };
            let cv = if v == 0 { 1.0 / sqrt8 } else { sqrt2 / sqrt8 };
            
            output[u][v] = cu * cv * sum;
        }
    }

    output
}

/// Level shift: subtract 128 from each pixel value
pub fn level_shift(block: &mut Block8x8) {
    for i in 0..8 {
        for j in 0..8 {
            block[i][j] -= 128.0;
        }
    }
}

/// Extract an 8x8 block from a channel at given position
pub fn extract_block(channel: &[f32], width: usize, x: usize, y: usize) -> Block8x8 {
    let mut block = [[0.0f32; 8]; 8];
    let height = channel.len() / width;
    
    for by in 0..8 {
        for bx in 0..8 {
            let px = x + bx;
            let py = y + by;
            
            if px < width && py < height {
                block[by][bx] = channel[py * width + px];
            } else {
                // Padding for edge blocks - use 128 (middle gray)
                block[by][bx] = 128.0;
            }
        }
    }
    
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dct_dc_only() {
        let mut input = [[128.0f32; 8]; 8];
        level_shift(&mut input);
        
        let output = forward_dct(&input);
        
        // DC coefficient should be exactly zero for zero input
        assert!(output[0][0].abs() < 0.01);
        
        // All AC coefficients should be near zero
        for i in 0..8 {
            for j in 0..8 {
                if i != 0 || j != 0 {
                    assert!(output[i][j].abs() < 0.01, 
                           "AC[{},{}] = {} (expected ~0)", i, j, output[i][j]);
                }
            }
        }
    }

    #[test]
    fn test_level_shift() {
        let mut block = [[128.0f32; 8]; 8];
        level_shift(&mut block);
        assert!((block[0][0] - 0.0).abs() < 0.01);
        assert!((block[7][7] - 0.0).abs() < 0.01);
    }
    
    #[test]
    fn test_extract_block() {
        let channel: Vec<f32> = (0..64).map(|x| x as f32).collect();
        let block = extract_block(&channel, 8, 0, 0);
        
        assert!((block[0][0] - 0.0).abs() < 0.01);
        assert!((block[0][7] - 7.0).abs() < 0.01);
        assert!((block[7][0] - 56.0).abs() < 0.01);
    }

    #[test]
    fn test_dct_constant_block() {
        // Test with a constant value
        let mut input = [[50.0f32; 8]; 8];
        level_shift(&mut input); // Now all -78.0
        
        let output = forward_dct(&input);
        
        // DC should be -78 * sqrt(64) = -78 * 8
        let expected_dc = -78.0 * 8.0;
        assert!((output[0][0] - expected_dc).abs() < 1.0);
        
        // AC coefficients should all be zero
        for i in 0..8 {
            for j in 0..8 {
                if i != 0 || j != 0 {
                    assert!(output[i][j].abs() < 0.1);
                }
            }
        }
    }
}
