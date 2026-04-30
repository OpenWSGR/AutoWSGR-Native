use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use strum::IntoEnumIterator;
use vessel_type::VesselType;

/// Build script: reads PNG templates from TemplateData/ directory and packs
/// them into templates.bin.
///
/// # Layout of templates.bin:
/// | templates_count: u32 | templates: [Template] |
///
/// # Layout of Template:
/// | ship_type: u8 | height: u8 | width: u8 | pixels: [u8] |
///
/// Filename convention: {VesselType}{index}.png (e.g. AP0.png, BB1.png)
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("templates.bin");
    let file = File::create(&dest_path).unwrap();

    let template_dir = Path::new("TemplateData");
    let mut writer = BufWriter::new(file);

    // Collect and sort PNG files for deterministic output
    let mut entries: Vec<_> = fs::read_dir(template_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let templates_count = entries.len() as u32;
    writer.write_all(&templates_count.to_ne_bytes()).unwrap();

    for entry in &entries {
        let filename = entry.file_name();
        let name = filename.to_string_lossy();

        // Extract ship type from filename: strip trailing digits and .png
        let ship_type_str: String = name
            .trim_end_matches(".png")
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();

        // Find matching VesselType
        let ship_type = VesselType::iter()
            .find(|vt| vt.as_ref() == ship_type_str)
            .unwrap_or_else(|| panic!("Unknown ship type in filename: {name}"));
        writer.write_all(&[ship_type as u8]).unwrap();

        // Read image as grayscale
        let img = image::open(entry.path()).unwrap();
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();

        writer.write_all(&[height as u8]).unwrap();
        writer.write_all(&[width as u8]).unwrap();

        // Write raw pixel data row by row
        for y in 0..height {
            for x in 0..width {
                let pixel = gray.get_pixel(x, y).0[0];
                writer.write_all(&[pixel]).unwrap();
            }
        }
    }
}
