// Import all dependencies
use image::ImageReader; // Load and manipulate images
use image::RgbaImage; // Load and manipulate images
use std::collections::HashSet; // Store unique colors
use std::collections::HashMap; // Remap logic
use std::io::{self, Write}; // Handles user input/output
use std::path::Path; // Manage filesystem paths

// Calculate luminance of pixels in the provided image
fn brightness(pixel: &[u8; 4]) -> f64{
    let r = pixel[0] as f64;
    let g = pixel[1] as f64;
    let b = pixel[2] as f64;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn main() {
    println!("Enter path of file: ");
    io::stdout().flush().unwrap(); 
    // Writes the given text to standard output.
    //flushes the buffer to write immediately. 
    //If flushing fails, unwrap causes a panic.

    let mut path = String::new();
    io::stdin().read_line(&mut path) 
        .expect("Failed to read line");
    let path = path.trim(); 
    // creates a new mutable variable - path
    // allows user input through standard input and stores input in variable
    // trims the whitespace from the user input to just have the path

    let img = ImageReader::open(path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");
    println!("Image loaded successfully.");
    // pulls the image into memory as an immutable variable
    // prints error message on failure but decodes the image on a success
    // stores the image as a DynamicImage in the variable
    // prints a success message to show progress is being made

    let rgba_image = img.to_rgba8();
    let raw_pixels = rgba_image.as_raw();
    let mut unique_pixels = HashSet::new();
    for chunk in raw_pixels.chunks_exact(4) {
        unique_pixels.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    // Converts the DynamicImage to an 8-bit RGBA image. The actual image type
    // is ImageBuffer<Rgba<u8>, Vec<u8>>. Then 'rgba_image.as_raw()' gives us
    // the raw byte array of the image which allows us to see the u8 values.
    // We take a HashSet of the array to only store unique values.
    // We iterate over the array to obtain the RGBA values as 4 byte chunks.
    // We use the formatting to insert the chunk correctly into the array.

    let mut sorted_pixels: Vec<[u8; 4]> = unique_pixels.into_iter().collect();
    sorted_pixels.sort_by(|a, b| brightness(b)
        .partial_cmp(&brightness(a))
        .unwrap());
    // Vec<[u8; 4]> is the formatting we set previously as we have raw byte data
    // The brightness function is called to sort by descending brightness
    // partial_cmp returns an Option, so we unwrap to panic in-case of errors

    let labels = [
        "Background White",
        "Highlight Bright",
        "Highlight Dark",
        "Midtone",
        "Shade Bright",
        "Shade Dark",
        "Outline",
        "Black",
    ];
    // These are labels that we will be assigning to the values of the imported image

    let mut remap: HashMap<[u8; 4], [u8; 4]> = HashMap::new();
    // we create a new mutable variable of remap which is a HashMap
    // we match a given pixel color [u8; 4] to another pixel color
    // then we create a new HashMap to store the mapped values

    for (i, pixel) in sorted_pixels.iter().enumerate() {
        // for loop to iterate over sorted pixels. we get the index(i)
        // and we also get the pixel value (pixel) in each iteration

        let label_owned = format!("Extra {}", i);
        let label = labels.get(i).copied().unwrap_or(&label_owned);
        let [r, g, b, a] = *pixel;
        // label_owned formats an additional label if there are more colors than
        // labels we have.
        // labels.get(i) gets the current iteration label
        // .copied() converts from &&stre to &str since labels is an array of strings
        // and is a reference to a reference. We take just the inner reference here
        // We then deconstruct the pixel into r, g, b and a for printing

        println!("\n{}: ({}, {}, {}, {})", label, r, g, b, a);
        print!("Enter new RGBA for this label, or leave blank to skip: ");
        io::stdout().flush().unwrap();
        // Prints the current label and rgba values on a new line for the user
        // we then prompt the user to enter in new values for the displayed rgba
        // values. We flush the buffer to make it display the text immediately

        let mut new_value = String::new();
        io::stdin().read_line(&mut new_value).unwrap();
        let new_value = new_value.trim();
        // we create a new mutable value that can store a string
        // we create a new standard input that stores the user input in the variable
        // like before we trim the whitespace to just have the values the provided
        // user vaules. We also unwrap in-case of any errors to exit the program

        if new_value.is_empty() {
            remap.insert(*pixel, *pixel);
            // if no input is given, we just map the original pixel value to itself
        } else {
            let parts: Vec<u8> = new_value
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            // if a value is given we make a new u8 (0-255) array and split on ','
            // each successfully parsed value is collected into the vector

            if parts.len() == 4 {
                // if the length of the part is 4

                remap.insert(*pixel, [parts[0], parts[1], parts[2], parts[3]]);
                // we map the provided values to the original values by inserting
                // them into the array

            } else {
                eprintln!("Invalid input, keeping original.");
                remap.insert(*pixel, *pixel);
                // if an error happens we just map the original to itself to keep
                // the same value
            }
        }
    }

    let mut output = RgbaImage::new(img.width(), img.height());
    // we use the RgbaImage alias for ImageBuffer<Rgba<u8>, Vec<u8>>
    // this is an image where each pixel is an rgba value
    // we create a new image buffer "new(img.width(), img.height())"
    // the buffer is the same size as the original as we reference the original
    // initially this buffer is all zeros, so black and fully transparent

    for (x, y, pixel) in rgba_image.enumerate_pixels() {
        // we create a for loop that iterates over every pixel in the image buffer
        // pixel is a reference to a Rgba<u8> struct which wraps the formatting
        // we've used before "[u8; 4]"

        let rgba = pixel.0;
        // we extract the raw array from the pixel using .0 which is the internal
        // array inside of the Rgba struct

        let new_rgba = remap.get(&rgba).unwrap_or(&rgba);
        // we look up the original color in the remap hash map
        // if a new color was specified we point to the replacement color
        // if not, we default to the original color (unwrap_or(&rgba)

        output.put_pixel(x, y, image::Rgba(*new_rgba));
        // writes the color value into the same coordinates in the new array
    }

    print!("\nEnter filename for modified file: ");
    io::stdout().flush().unwrap();
    let mut output_name = String::new();
    io::stdin().read_line(&mut output_name).unwrap();
    let output_name = output_name.trim();
    // we prompt for a new file name, clearing the buffer to print immediately
    // make a new mutable variable to accept a string
    // and then store the user input in the variable, trimming the value

    let save_path = Path::new(path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(output_name);
    // we take the location of the original image and convert it to a path object
    // .parent() gets the directory of the original file
    // .join(output_name) appends the new filename the user entered into the path
    // this way we have a full UNC path to tell the OS where to store the file

    output
        .save(&save_path)
        .expect("Failed to save output image");
    // we save the buffer to disk in the bitmap image
    // if something goes wrong with saving, we panic to terminate the program

    println!("Image saved as: {}", save_path.display());
    // displays the path of the newly saved file

    }
