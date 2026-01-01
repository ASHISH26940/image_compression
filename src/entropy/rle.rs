#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RLESymbol {
    pub run_length: u8,  // 0-15 zeros before value
    pub value: i32,      // The actual coefficient
}

pub const EOB:RLESymbol=RLESymbol{
    run_length:0,value:0
};

pub const ZRL:RLESymbol=RLESymbol{run_length:15,value:0};

pub fn encode_dc_diff(current_dc:i32,previous_dc:i32)->i32{
    current_dc-previous_dc
}

pub fn run_length_encode_ac(zigzag:&[i32])->Vec<RLESymbol>{
    let mut result=Vec::new();
    let mut zero_run=0u8;

    for &coeff in &zigzag[1..]{
        if coeff == 0{
             zero_run+=1;

             if zero_run ==16{
                result.push(ZRL);
                zero_run=0;
             }
        }else{
            result.push(RLESymbol { run_length: zero_run, value: coeff });
            zero_run=0;
        }
    }

    if zero_run >0{
        result.push(EOB);
    }
    
    result
}

pub fn get_size_category(value: i32) -> u8 {
    if value == 0 {
        return 0;
    }
    
    let abs_val = value.abs();
    let bits = 32 - abs_val.leading_zeros();
    bits as u8
}

pub fn get_amplitude_bits(value: i32, size: u8) -> u16 {
    if value > 0 {
        value as u16
    } else {
        // For negative values, use 1's complement: (2^size - 1) + value
        ((1 << size) - 1 + value) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_dpcm() {
        assert_eq!(encode_dc_diff(150, 0), 150);
        assert_eq!(encode_dc_diff(155, 150), 5);
        assert_eq!(encode_dc_diff(149, 155), -6);
        assert_eq!(encode_dc_diff(152, 149), 3);
    }

    #[test]
    fn test_rle_simple() {
        let zigzag = vec![42, 12, 0, 0, 5, 0, 0, 0];
        let rle = run_length_encode_ac(&zigzag);
        
        assert_eq!(rle.len(), 3);
        assert_eq!(rle[0], RLESymbol { run_length: 0, value: 12 });
        assert_eq!(rle[1], RLESymbol { run_length: 2, value: 5 });
        assert_eq!(rle[2], EOB);
    }

    #[test]
    fn test_rle_with_zrl() {
        // 20 consecutive zeros
        let mut zigzag = vec![42]; // DC
        zigzag.extend(vec![0; 20]);
        zigzag.push(7); // Non-zero at the end
        
        let rle = run_length_encode_ac(&zigzag);
        
        // Should have: ZRL (16 zeros), then (4, 7)
        assert_eq!(rle[0], ZRL);
        assert_eq!(rle[1], RLESymbol { run_length: 4, value: 7 });
    }

    #[test]
    fn test_size_category() {
        assert_eq!(get_size_category(0), 0);
        assert_eq!(get_size_category(1), 1);
        assert_eq!(get_size_category(-1), 1);
        assert_eq!(get_size_category(7), 3);
        assert_eq!(get_size_category(-7), 3);
        assert_eq!(get_size_category(255), 8);
    }

    #[test]
    fn test_amplitude_bits() {
        assert_eq!(get_amplitude_bits(5, 3), 0b101);
        assert_eq!(get_amplitude_bits(-5, 3), 0b010); // 1's complement
        assert_eq!(get_amplitude_bits(1, 1), 0b1);
        assert_eq!(get_amplitude_bits(-1, 1), 0b0);
    }
}