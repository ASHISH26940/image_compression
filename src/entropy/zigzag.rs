pub const ZIGZAG_ORDER: [(usize, usize); 64] = [
    (0,0), (0,1), (1,0), (2,0), (1,1), (0,2), (0,3), (1,2),
    (2,1), (3,0), (4,0), (3,1), (2,2), (1,3), (0,4), (0,5),
    (1,4), (2,3), (3,2), (4,1), (5,0), (6,0), (5,1), (4,2),
    (3,3), (2,4), (1,5), (0,6), (0,7), (1,6), (2,5), (3,4),
    (4,3), (5,2), (6,1), (7,0), (7,1), (6,2), (5,3), (4,4),
    (3,5), (2,6), (1,7), (2,7), (3,6), (4,5), (5,4), (6,3),
    (7,2), (7,3), (6,4), (5,5), (4,6), (3,7), (4,7), (5,6),
    (6,5), (7,4), (7,5), (6,6), (5,7), (6,7), (7,6), (7,7),
];

pub fn zig_zag(block:&[[i32;8];8])->Vec<i32>{
    let mut output=Vec::with_capacity(64);

    for &(row,col) in ZIGZAG_ORDER.iter(){
        output.push(block[row][col]);
    }

    output
}

pub fn inverse_zigzag(data: &[i32; 64]) -> [[i32; 8]; 8]{
    let mut block = [[0i32; 8]; 8];
    
    for (i, &(row, col)) in ZIGZAG_ORDER.iter().enumerate() {
        block[row][col] = data[i];
    }
    
    block
}

