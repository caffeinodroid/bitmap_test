// Import all dependencies
use image::ImageReader; // Load and manipulate images
use image::RgbaImage; // Load and manipulate images
use std::collections::HashSet; // Store unique colors
use std::collections::HashMap; // Remap logic
use std::io::{self, Write}; // Handles user input/output
use std::path::Path; // Manage filesystem paths
use rfd::FileDialog;

// Calculate luminance of pixels in the provided image
fn brightness(pixel: &[u8; 4]) -> f64{
    let r = pixel[0] as f64;
    let g = pixel[1] as f64;
    let b = pixel[2] as f64;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn load_image(path: &Path) -> RgbaImage {
    ImageReader::open(path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image")
        .to_rgba8()
}

fn prompt_mode() -> String {
    println!("\nChoose mode:");
    println!("1 - Change a single color by label");
    println!("2 - Change all defined colors");
    print!("Enter 1 or 2: ");
    io::stdout().flush().unwrap();

    let mut mode = String::new();
    io::stdin().read_line(&mut mode).unwrap();
    mode.trim().to_string()
}

fn extract_labeled_colors(image: &RgbaImage, labels: &[&str]) -> (Vec<[u8; 4]>, HashMap<String, [u8; 4]>) {
    let mut unique_pixels = HashSet::new();

    for chunk in image.as_raw().chunks_exact(4) {
        unique_pixels.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    let mut sorted_pixels: Vec<[u8; 4]> = unique_pixels.into_iter().collect();
    sorted_pixels.sort_by(|a, b| brightness(b).partial_cmp(&brightness(a)).unwrap());

    let mut label_map: HashMap<String, [u8; 4]> = HashMap::new();
    for (i, pixel) in sorted_pixels.iter().enumerate() {
        let label_owned = format!("Extra {}", i);
        let label = labels.get(i).copied().unwrap_or(&label_owned);
        label_map.insert(label.to_string(), *pixel);
    }

    (sorted_pixels, label_map)
}

fn remap_colors(mode: &str, _sorted_pixels: &[ [u8; 4] ], label_map: &HashMap<String, [u8; 4]>) -> HashMap<[u8; 4], [u8; 4]> {
    let mut remap = HashMap::new();

    match mode {
        "1" => {
            println!("Available labels:");
            for label in label_map.keys() {
                println!("- {}", label);
            }

            print!("Enter label to change: ");
            io::stdout().flush().unwrap();
            let mut label_input = String::new();
            io::stdin().read_line(&mut label_input).unwrap();
            let label_input = label_input.trim();

            match label_map.get(label_input) {
                Some(&old_color) => {
                    print!("Enter new RGBA (R,G,B,A): ");
                    io::stdout().flush().unwrap();
                    let mut new_value = String::new();
                    io::stdin().read_line(&mut new_value).unwrap();
                    let parts: Vec<u8> = new_value
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u8>().ok())
                        .collect();

                    match parts.as_slice() {
                        [r, g, b, a] => {
                            remap.insert(old_color, [*r, *g, *b, *a]);
                        }
                        _ => {
                            eprintln!("Invalid input. Skipping remap.");
                            remap.insert(old_color, old_color);
                        }
                    }
                }
                None => {
                    eprintln!("Label not found.");
                }
            }
        }
        "2" => {
            for (label, &pixel) in label_map {
                let [r, g, b, a] = pixel;
                println!("\n{}: ({}, {}, {}, {})", label, r, g, b, a);
                print!("Enter new RGBA for this label, or leave blank to skip: ");
                io::stdout().flush().unwrap();

                let mut new_value = String::new();
                io::stdin().read_line(&mut new_value).unwrap();
                let new_value = new_value.trim();

                if new_value.is_empty() {
                    remap.insert(pixel, pixel);
                } else {
                    let parts: Vec<u8> = new_value
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u8>().ok())
                        .collect();

                    if parts.len() == 4 {
                        remap.insert(pixel, [parts[0], parts[1], parts[2], parts[3]]);
                    } else {
                        eprintln!("Invalid input, keeping original.");
                        remap.insert(pixel, pixel);
                    }
                }
            }
        }
        _ => {
            eprintln!("Invalid mode selected.");
        }
    }

    remap
}

fn main() {
    let labels = [
        "white",
        "bright_highlight",
        "dark_highlight",
        "midtone",
        "bright_shade",
        "dark_shade",
        "outline",
        "black",
    ];

    let image_paths: Vec<std::path::PathBuf> = FileDialog::new()
        .add_filter("Image files", &["png", "jpg", "jpeg", "bmp"])
        .set_title("Select one or more images to modify")
        .pick_files()
        .unwrap_or_else(|| {
            println!("No files selected.");
            std::process::exit(1);
        });

    for path in image_paths {
        let rgba_image = load_image(&path);
        let (sorted_pixels, label_map) = extract_labeled_colors(&rgba_image, &labels);
        let mode = prompt_mode();
        let remap = remap_colors(&mode, &sorted_pixels, &label_map);

        let mut output = RgbaImage::new(rgba_image.width(), rgba_image.height());
        for (x, y, pixel) in rgba_image.enumerate_pixels() {
            let rgba = pixel.0;
            let new_rgba = remap.get(&rgba).unwrap_or(&rgba);
            output.put_pixel(x, y, image::Rgba(*new_rgba));
        }

        print!("\nEnter filename for modified file: ");
        io::stdout().flush().unwrap();
        let mut output_name = String::new();
        io::stdin().read_line(&mut output_name).unwrap();
        let output_name = output_name.trim();

        let save_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(output_name);
        output.save(&save_path).expect("Failed to save output image");

        println!("✅ Image saved as: {}", save_path.display());
    }
}
