use std::iter::Filter;

use crate::png::filter;

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

pub fn filter_scanline(
    scanline: &[u8],
    prev_scanline: Option<&[u8]>,
    filter_type: FilterType,
    bytes_per_pixel: usize,
) -> Vec<u8> {
    let mut filtered=Vec::with_capacity(scanline.len()+1);
    filtered.push(filter_type as u8);

    match filter_type {
        FilterType::None => {
            filtered.extend_from_slice(scanline);
        }
        FilterType::Sub => {
            filter_sub(scanline, &mut filtered, bytes_per_pixel);
        }
        FilterType::Up => {
            filter_up(scanline, prev_scanline.unwrap_or(&[]), &mut filtered);
        }
        FilterType::Average => {
            filter_average(scanline, prev_scanline.unwrap_or(&[]), &mut filtered, bytes_per_pixel);
        }
        FilterType::Paeth => {
            filter_paeth(scanline, prev_scanline.unwrap_or(&[]), &mut filtered, bytes_per_pixel);
        }
    }
    
    
    filtered
}


// Filter Type 1: Sub (subtract left pixel)
fn filter_sub(scanline: &[u8], output: &mut Vec<u8>, bpp: usize) {
    for i in 0..scanline.len() {
        let left = if i >= bpp { scanline[i - bpp] } else { 0 };
        output.push(scanline[i].wrapping_sub(left));
    }
}

/// Filter Type 2: Up (subtract pixel above)
fn filter_up(scanline: &[u8], prev: &[u8], output: &mut Vec<u8>) {
    for i in 0..scanline.len() {
        let up = if i < prev.len() { prev[i] } else { 0 };
        output.push(scanline[i].wrapping_sub(up));
    }
}

/// Filter Type 3: Average
fn filter_average(scanline: &[u8], prev: &[u8], output: &mut Vec<u8>, bpp: usize) {
    for i in 0..scanline.len() {
        let left = if i >= bpp { scanline[i - bpp] } else { 0 };
        let up = if i < prev.len() { prev[i] } else { 0 };
        let avg = ((left as u16 + up as u16) / 2) as u8;
        output.push(scanline[i].wrapping_sub(avg));
    }
}

/// Filter Type 4: Paeth predictor
fn filter_paeth(scanline: &[u8], prev: &[u8], output: &mut Vec<u8>, bpp: usize) {
    for i in 0..scanline.len() {
        let left = if i >= bpp { scanline[i - bpp] } else { 0 };
        let up = if i < prev.len() { prev[i] } else { 0 };
        let upper_left = if i >= bpp && i < prev.len() { prev[i - bpp] } else { 0 };
        
        let predictor = paeth_predictor(left, up, upper_left);
        output.push(scanline[i].wrapping_sub(predictor));
    }
}

/// Paeth predictor algorithm
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

/// Choose best filter for a scanline (simple heuristic)
pub fn choose_best_filter(
    scanline: &[u8],
    prev_scanline: Option<&[u8]>,
    bytes_per_pixel: usize,
) -> FilterType {
    let filters = [
        FilterType::None,
        FilterType::Sub,
        FilterType::Up,
        FilterType::Average,
        FilterType::Paeth,
    ];
    
    let mut best_filter = FilterType::None;
    let mut best_score = u64::MAX;
    
    for &filter in &filters {
        let filtered = filter_scanline(scanline, prev_scanline, filter, bytes_per_pixel);
        
        // Score: sum of absolute values (smaller is better)
        let score: u64 = filtered[1..].iter().map(|&b| b.abs_diff(128) as u64).sum();
        
        if score < best_score {
            best_score = score;
            best_filter = filter;
        }
    }
    
    best_filter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_none() {
        let data = vec![100, 120, 140];
        let filtered = filter_scanline(&data, None, FilterType::None, 1);
        assert_eq!(filtered, vec![0, 100, 120, 140]);
    }

    #[test]
    fn test_filter_sub() {
        let data = vec![100, 102, 105];
        let filtered = filter_scanline(&data, None, FilterType::Sub, 1);
        assert_eq!(filtered[0], 1); // Filter type
        assert_eq!(filtered[1], 100); // First byte unchanged
        assert_eq!(filtered[2], 2);   // 102 - 100 = 2
        assert_eq!(filtered[3], 3);   // 105 - 102 = 3
    }

    #[test]
    fn test_paeth_predictor() {
        assert_eq!(paeth_predictor(10, 20, 15), 10);
        assert_eq!(paeth_predictor(50, 50, 50), 50);
    }
}