
use std::env;
use std::fs;
use std::path::Path;
use jpeg_encoder::{JpegEncoder, PngEncoder};
use image::GenericImageView;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
    Jpeg,
    Png,
}

fn main() {
    println!("=== Image Encoder v{} ===\n", jpeg_encoder::VERSION);
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        println!("\n💡 No input file specified. Generating test images...\n");
        generate_test_images();
        return;
    }
    
    let input_path = &args[1];
    
    // Parse format (--format jpeg|png)
    let format = parse_format(&args);
    
    // Parse quality (for JPEG only)
    let quality = if format == OutputFormat::Jpeg {
        parse_quality(&args)
    } else {
        85 // Ignored for PNG
    };
    
    // Parse compression level (for PNG only, 0-9)
    let compression = if format == OutputFormat::Png {
        parse_compression(&args)
    } else {
        6 // Default
    };
    
    // Generate output path
    let output_path = generate_output_path(input_path, &args, format);
    
    // Compress the image
    match compress_image(input_path, &output_path, format, quality, compression) {
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
    println!("📷 Image Compression Tool (JPEG & PNG)");
    println!("\nUsage:");
    println!("  image_encoder <input_file> [--format FORMAT] [--quality Q] [--compression C] [--output FILE]");
    println!();
    println!("Options:");
    println!("  --format FORMAT     Output format: jpeg or png (default: jpeg)");
    println!("  --quality Q         JPEG quality 1-100 (default: 85, only for JPEG)");
    println!("  --compression C     PNG compression 0-9 (default: 6, only for PNG)");
    println!("  --output FILE       Output filename (default: auto-generated)");
    println!();
    println!("Examples:");
    println!("  # JPEG compression");
    println!("  image_encoder photo.png --format jpeg --quality 90");
    println!("  ");
    println!("  # PNG compression (lossless)");
    println!("  image_encoder photo.jpg --format png --compression 9");
    println!("  ");
    println!("  # Quick JPEG (default)");
    println!("  image_encoder photo.png");
}

fn parse_format(args: &[String]) -> OutputFormat {
    for i in 0..args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            match args[i + 1].to_lowercase().as_str() {
                "png" => return OutputFormat::Png,
                "jpeg" | "jpg" => return OutputFormat::Jpeg,
                _ => {}
            }
        }
    }
    OutputFormat::Jpeg // Default
}

fn parse_quality(args: &[String]) -> u8 {
    for i in 0..args.len() {
        if args[i] == "--quality" && i + 1 < args.len() {
            if let Ok(q) = args[i + 1].parse::<u8>() {
                return q.clamp(1, 100);
            }
        }
    }
    85 // Default
}

fn parse_compression(args: &[String]) -> u32 {
    for i in 0..args.len() {
        if args[i] == "--compression" && i + 1 < args.len() {
            if let Ok(c) = args[i + 1].parse::<u32>() {
                return c.min(9);
            }
        }
    }
    6 // Default
}

fn generate_output_path(input: &str, args: &[String], format: OutputFormat) -> String {
    // Check for explicit --output
    for i in 0..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    
    // Auto-generate based on format
    let path = Path::new(input);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    
    match format {
        OutputFormat::Jpeg => format!("{}_compressed.jpg", stem),
        OutputFormat::Png => format!("{}_compressed.png", stem),
    }
}

fn compress_image(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: u8,
    compression: u32,
) -> Result<(), String> {
    println!("📂 Reading image: {}", input_path);
    
    if !Path::new(input_path).exists() {
        return Err(format!("File not found: {}", input_path));
    }
    
    let img = image::open(input_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;
    
    let (width, height) = img.dimensions();
    let color_type = img.color();
    
    println!("   📐 Dimensions: {}x{} pixels", width, height);
    println!("   🎨 Color type: {:?}", color_type);
    
    let original_file_size = fs::metadata(input_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);
    
    println!("   💾 Original file: {} bytes ({:.2} MB)", 
             original_file_size, 
             original_file_size as f64 / 1_048_576.0);
    
    // Encode based on format
    let compressed_data = match format {
        OutputFormat::Jpeg => {
            println!("\n🔄 Encoding to JPEG (quality {})...", quality);
            let rgb_img = img.to_rgb8();
            let rgb_pixels = rgb_img.as_raw();
            
            let encoder = JpegEncoder::new(width as u16, height as u16, quality);
            encoder.encode(rgb_pixels)?
        }
        OutputFormat::Png => {
            println!("\n🔄 Encoding to PNG (compression level {})...", compression);
            let rgb_img = img.to_rgb8();
            let rgb_pixels = rgb_img.as_raw();
            
            let encoder = PngEncoder::new(width, height).with_compression(compression);
            encoder.encode_rgb(rgb_pixels)?
        }
    };
    
    // Save output
    println!("💾 Saving to: {}", output_path);
    fs::write(output_path, &compressed_data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    // Show compression statistics
    let compressed_size = compressed_data.len();
    let ratio = original_file_size as f64 / compressed_size as f64;
    let savings = 100.0 - (compressed_size as f64 / original_file_size as f64 * 100.0);
    
    println!("\n📊 Compression Results:");
    println!("   Original file:     {:>10} bytes ({:>6.2} MB)", 
             original_file_size, 
             original_file_size as f64 / 1_048_576.0);
    println!("   Compressed file:   {:>10} bytes ({:>6.2} MB)", 
             compressed_size,
             compressed_size as f64 / 1_048_576.0);
    println!("   Compression ratio: {:>10.2}:1", ratio);
    
    if savings > 0.0 {
        println!("   Space saved:       {:>10.1}% smaller", savings);
    } else {
        println!("   Space saved:       {:>10.1}% larger", -savings);
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
