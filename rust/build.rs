use std::collections::HashSet;
use std::env::{self, current_dir};
use std::ffi::OsStr;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::str::from_utf8;

use image::{GenericImageView, Rgba};
use proc_macro2::{Span, Ident, TokenStream};
use quote::quote;

fn main() {    
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Run Python script to build fonts
    // (Script output is the source of a Rust module named `font_data`)
    let output = Command::new("python3")
        .arg("../build_font.py")
        .arg(current_dir().unwrap())
        .output()
        .expect("Python invocation to build fonts failed");
    if !output.status.success() {
        panic!(
            "Python invocation to build fonts failed:\n\nstdout:\n{}\n\nstderr:\n{}",
            from_utf8(&output.stdout[..]).unwrap(),
            from_utf8(&output.stderr[..]).unwrap(),
        );
    }

    // Write fonts to file
    fs::write(
        out.join("font_data.rs"),
        from_utf8(&output.stdout[..]).unwrap()
    ).unwrap();

    // Generate bitmap data from images
    let mut bitmap_sources = vec![];
    let mut bitmap_constant_names = vec![];
    let images = fs::read_dir("../res")
        .unwrap()
        .into_iter()
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension() == Some(OsStr::new("png")));
    for image in images {
        bitmap_sources.push(bitmap_rust_source(&image));
        bitmap_constant_names.push(bitmap_path_to_name(&image));
    }
    bitmap_sources.push(bitmap_lookup_rust_source(&bitmap_constant_names[..]));

    let bitmap_source = quote! {
        pub mod bitmap_data {
            #( #bitmap_sources )*
        }
    };

    // Write bitmaps to file
    fs::write(out.join("bitmap_data.rs"), bitmap_source.to_string()).unwrap();

    // TODO: Rerun only if changed
}

fn bitmap_path_to_name(image_path: &Path) -> String {
    image_path.file_stem().unwrap().to_ascii_uppercase().into_string().unwrap()
}

fn bitmap_rust_source(image_path: &Path) -> TokenStream {
    let data = bitmap_data(image_path);

    let name = Ident::new(&bitmap_path_to_name(image_path), Span::call_site());
    let length = data.len();

    quote! {
        pub const #name: [u16; #length] = [ #(#data),* ];
    }
}

fn bitmap_data(image_path: &Path) -> Vec<u16> {
    const PLACEHOLDER: u16 = 0;

    let img = image::open(image_path).unwrap();

    // Build a set of all colours in the image, so we can find a colour we aren't using
    // An unused colour will become a transparency colour
    // (This assumes that it won't use _every_ 16-bit colour!)
    let mut used_colours = HashSet::new();

    // Get dimensions
    let width = img.width();
    let height = img.height();

    // Start building up image data, keeping track of transparency and run-length marker indexes to
    // be replaced
    let mut data = vec![width as u16, height as u16, 0, 0];
    let mut transparency_indexes = vec![2];
    let mut run_length_indexes = vec![3];

    for x in 0..width {
        let mut y = 0;
        while y < height {
            // Count how many consecutive pixels on this row use this colour
            let colour = colour_8888_to_565(img.get_pixel(x, y));
            let mut count = 1;
            y += 1;

            while y < height && count < u16::MAX {
                if colour_8888_to_565(img.get_pixel(x, y)) == colour {
                    count += 1;
                    y += 1;
                } else {
                    break;
                }
            }

            // If this is non-transparent, add to used colours
            if let Some(colour) = colour {
                used_colours.insert(colour);
            }

            // If there are more than four pixels of the same colour, it's more space efficient to
            // use run-length encoding
            if count > 4 {
                // Run-length marker and count
                run_length_indexes.push(data.len());
                data.push(PLACEHOLDER); 
                data.push(count);

                // Colour
                if let Some(colour) = colour {
                    data.push(colour);
                } else {
                    transparency_indexes.push(data.len());
                    data.push(PLACEHOLDER);
                }
            } else {
                // Just output each colour
                for _ in 0..count {
                    if let Some(colour) = colour {
                        data.push(colour);
                    } else {
                        transparency_indexes.push(data.len());
                        data.push(PLACEHOLDER);
                    }
                }
            }
        }
    }

    // Find unused colours to use as transparency and run-length markers
    let two_unused_colours = (0..u16::MAX)
        .into_iter()
        .filter(|x| !used_colours.contains(x))
        .take(2)
        .collect::<Vec<_>>();
    if two_unused_colours.len() < 2 {
        panic!(
            "image {:?} uses too many colours to convert to a bitmap - at least two RGB565 colours must be available",
            image_path
        );
    }
    let transparency_colour = two_unused_colours[0];
    let run_length_colour = two_unused_colours[1];

    // Replace marked locations with these colours
    for i in transparency_indexes {
        data[i] = transparency_colour;
    }
    for i in run_length_indexes {
        data[i] = run_length_colour;
    }

    data
}

fn colour_8888_to_565(colour: Rgba<u8>) -> Option<u16> {
    // Conversion: http://www.barth-dev.de/online/rgb565-color-picker/
    let [red, green, blue, alpha] = colour.0;

    // Arbitrary transparency threshold
    if alpha > 200 {
        Some(
            ((red as u16 & 0b11111000) << 8) +
            ((green as u16 & 0b11111100) << 3) +
            (blue as u16 >> 3)
        )
    } else {
        None
    }
}

fn bitmap_lookup_rust_source(constant_names: &[String]) -> TokenStream {
    let cases = constant_names.iter().map(|name| {
        let name_as_ident = Ident::new(name, Span::call_site());
        quote! {
            #name => & #name_as_ident [..]
        }
    });

    quote! {
        pub fn lookup(name: &str) -> &'static [u16] {
            match name {
                #( #cases ),* ,

                _ => panic!("unknown bitmap"),
            }
        }
    }
}
