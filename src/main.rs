//! JPEG Encoder CLI - Compress any image to JPEG

use std::env;
use std::fs;
use std::path::Path;
use jpeg_encoder::JpegEncoder;
use image::GenericImageView;

fn main() {
    println!("=== JPEG Encoder v{} ===\n", jpeg_encoder::VERSION);
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        
        // If no arguments, generate test images
        println!("\n💡 No input file specified. Generating test images...\n");
        generate_test_images();
        return;
    }
    
    let input_path = &args[1];
    
    // Parse quality (default: 85)
    let quality = if args.len() >= 3 {
        args[2].parse::<u8>().unwrap_or(85).clamp(1, 100)
    } else {
        85
    };
    
    // Parse output path (default: auto-generate)
    let output_path = if args.len() >= 4 {
        args[3].clone()
    } else {
        auto_output_filename(input_path)
    };
    
    // Compress the image
    match compress_image(input_path, &output_path, quality) {
        Ok(_) => {
            println!("\n✅ Success! Compressed image saved to: {}", output_path);
            println!("\nYou can now open {} in any image viewer!", output_path);
        }
        Err(e) => {
            println!("\n❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("📷 JPEG Image Compression Tool");
    println!("\nUsage:");
    println!("  jpeg_encoder_cli <input_file> [quality] [output_file]");
    println!();
    println!("Arguments:");
    println!("  input_file   - Input image (JPEG, PNG, BMP, GIF, TIFF, WebP, etc.)");
    println!("  quality      - JPEG quality 1-100 (default: 85)");
    println!("                 1 = smallest file, lowest quality");
    println!("                 100 = largest file, highest quality");
    println!("  output_file  - Output JPEG file (default: <input>_compressed.jpg)");
    println!();
    println!("Examples:");
    println!("  jpeg_encoder_cli photo.png");
    println!("  jpeg_encoder_cli photo.png 90");
    println!("  jpeg_encoder_cli photo.png 75 result.jpg");
    println!("  jpeg_encoder_cli existing.jpg 50 smaller.jpg");
}

fn auto_output_filename(input_path: &str) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    format!("{}_compressed.jpg", stem)
}

fn compress_image(input_path: &str, output_path: &str, quality: u8) -> Result<(), String> {
    println!("📂 Reading image: {}", input_path);
    
    // Check if file exists
    if !Path::new(input_path).exists() {
        return Err(format!("File not found: {}", input_path));
    }
    
    // Load the image using the image crate
    let img = image::open(input_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;
    
    let (width, height) = img.dimensions();
    let color_type = img.color();
    
    println!("   📐 Dimensions: {}x{} pixels", width, height);
    println!("   🎨 Color type: {:?}", color_type);
    
    // Convert to RGB8 (our encoder expects RGB)
    let rgb_img = img.to_rgb8();
    let rgb_pixels = rgb_img.as_raw();
    
    let original_rgb_size = rgb_pixels.len();
    let original_file_size = fs::metadata(input_path)
        .map(|m| m.len() as usize)
        .unwrap_or(original_rgb_size);
    
    println!("   💾 Original file: {} bytes ({:.2} MB)", 
             original_file_size, 
             original_file_size as f64 / 1_048_576.0);
    println!("   💾 Uncompressed RGB: {} bytes ({:.2} MB)", 
             original_rgb_size,
             original_rgb_size as f64 / 1_048_576.0);
    
    // Encode to JPEG
    println!("\n🔄 Encoding to JPEG (quality {})...", quality);
    let encoder = JpegEncoder::new(width as u16, height as u16, quality);
    
    let jpeg_data = encoder.encode(rgb_pixels)
        .map_err(|e| format!("Encoding failed: {}", e))?;
    
    // Save the output
    println!("💾 Saving to: {}", output_path);
    fs::write(output_path, &jpeg_data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    // Show compression statistics
    let compressed_size = jpeg_data.len();
    let ratio = original_file_size as f64 / compressed_size as f64;
    let rgb_ratio = original_rgb_size as f64 / compressed_size as f64;
    let savings = 100.0 - (compressed_size as f64 / original_file_size as f64 * 100.0);
    
    println!("\n📊 Compression Results:");
    println!("   Original file:     {:>10} bytes ({:>6.2} MB)", 
             original_file_size, 
             original_file_size as f64 / 1_048_576.0);
    println!("   Compressed JPEG:   {:>10} bytes ({:>6.2} MB)", 
             compressed_size,
             compressed_size as f64 / 1_048_576.0);
    println!("   Compression ratio: {:>10.2}:1", ratio);
    println!("   RGB compression:   {:>10.2}:1", rgb_ratio);
    
    if savings > 0.0 {
        println!("   Space saved:       {:>10.1}% smaller", savings);
    } else {
        println!("   Space saved:       {:>10.1}% larger (quality too high)", -savings);
    }
    
    Ok(())
}

fn generate_test_images() {
    println!("🎨 Generating test images...\n");
    
    test_gradient_image();
    test_color_bars();
    test_checkerboard();
    test_simple_gray();
    
    println!("\n✅ Generated test images:");
    println!("   - gradient.jpg (smooth color gradient)");
    println!("   - colorbars.jpg (TV test pattern)");
    println!("   - checkerboard.jpg (high-frequency pattern)");
    println!("   - test_gray.jpg (solid gray square)");
    println!("\n💡 Now try: jpeg_encoder_cli <your_image.png> 85");
}

fn test_simple_gray() {
    let width = 8;
    let height = 8;
    let pixels = vec![128u8; 8 * 8 * 3];
    encode_and_save("test_gray.jpg", &pixels, width, height, 50);
}

fn test_gradient_image() {
    let width = 256;
    let height = 256;
    let mut pixels = vec![0u8; width * height * 3];
    
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            pixels[idx] = (x * 255 / width) as u8;
            pixels[idx + 1] = (y * 255 / height) as u8;
            pixels[idx + 2] = ((x + y) * 255 / (width + height)) as u8;
        }
    }
    
    encode_and_save("gradient.jpg", &pixels, width, height, 85);
}

fn test_color_bars() {
    let width = 280;
    let height = 200;
    let mut pixels = vec![0u8; width * height * 3];
    
    let colors = [
        (255, 255, 255), (255, 255, 0), (0, 255, 255), (0, 255, 0),
        (255, 0, 255), (255, 0, 0), (0, 0, 255),
    ];
    
    let bar_width = width / colors.len();
    
    for y in 0..height {
        for x in 0..width {
            let bar_index = (x / bar_width).min(colors.len() - 1);
            let (r, g, b) = colors[bar_index];
            
            let idx = (y * width + x) * 3;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
        }
    }
    
    encode_and_save("colorbars.jpg", &pixels, width, height, 90);
}

fn test_checkerboard() {
    let width = 256;
    let height = 256;
    let square_size = 32;
    let mut pixels = vec![0u8; width * height * 3];
    
    for y in 0..height {
        for x in 0..width {
            let is_white = ((x / square_size) + (y / square_size)) % 2 == 0;
            let value = if is_white { 255 } else { 0 };
            
            let idx = (y * width + x) * 3;
            pixels[idx] = value;
            pixels[idx + 1] = value;
            pixels[idx + 2] = value;
        }
    }
    
    encode_and_save("checkerboard.jpg", &pixels, width, height, 75);
}

fn encode_and_save(filename: &str, pixels: &[u8], width: usize, height: usize, quality: u8) {
    print!("   Creating {}... ", filename);
    
    let encoder = JpegEncoder::new(width as u16, height as u16, quality);
    
    match encoder.encode(pixels) {
        Ok(jpeg_data) => {
            match fs::write(filename, &jpeg_data) {
                Ok(_) => println!("✓ {} bytes", jpeg_data.len()),
                Err(e) => println!("✗ Failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }
}
