mod tables;

pub use tables::{
    STANDARD_LUMA_QTABLE, 
    STANDARD_CHROMA_QTABLE, 
    scale_quantization_table
};

pub fn quantize(dict_block:&[[f32;8];8],qtable:&[[f32;8];8])->[[i32;8];8]{
    let mut quantized=[[0i32;8];8];

    for i in 0..8{
        for j in 0..8{
            quantized[i][j]=(dict_block[i][j] / qtable[i][j]).round() as i32;
        }
    }

    quantized
}

pub fn dequantize(quantized: &[[i32; 8]; 8], qtable: &[[f32; 8]; 8]) -> [[f32; 8]; 8] {
    let mut dct_block=[[0.0f32;8];8];

    for i in 0..8{
        for j in 0..8{
            dct_block[i][j]=quantized[i][j] as f32 * qtable[i][j];
        }
    }

    dct_block
}