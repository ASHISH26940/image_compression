
#[derive(Debug,Clone,Copy)]
pub struct YCbCr{
    pub y:f32,
    pub cb:f32,
    pub cr:f32,
}

pub fn rgb_to_ycbcr(
    r:u8,
    g:u8,
    b:u8
)->YCbCr{
    let r=r as f32;
    let g= g as f32;
    let b=b as f32;

    YCbCr {
        y:  0.299 * r + 0.587 * g + 0.114 * b,
        cb: -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0,
        cr: 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0,
    }
}

pub fn rgb_image_to_ycbcr(rgb_pixels:&[u8],width:usize,height:usize)->(Vec<f32>,Vec<f32>,Vec<f32>){
    let pixel_count=width*height;
    let mut y_channel=Vec::with_capacity(pixel_count);
    let mut cb_channel=Vec::with_capacity(pixel_count);
    let mut cr_channel=Vec::with_capacity(pixel_count);

    for i in 0..pixel_count{
        let r=rgb_pixels[i*3];
        let g=rgb_pixels[i*3+1];
        let b=rgb_pixels[i*3+2];

        let ycbcr=rgb_to_ycbcr(r, g, b);
        y_channel.push(ycbcr.y);
        cb_channel.push(ycbcr.cb);
        cr_channel.push(ycbcr.cr);
    }

    (y_channel,cb_channel,cr_channel)
}