#![allow(unused_imports)]
extern crate image;
use ::image::{DynamicImage, ImageReader};

use crate::WrappedPixels;
use crate::image::BGRImage;
use crate::locator::*;
use crate::recognize_enemy;
use crate::recognize_enemy::character_image::CharacterImage;
use crate::recognize_enemy::templates;
use crate::recognize_enemy::*;
use crate::recognize_map::*;
struct ImageWrapper {
    width: usize,
    height: usize,
    channels: usize,
    pixels: Vec<u8>,
}
fn bgr_wrapper(image: &DynamicImage) -> ImageWrapper {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut pixels = image.to_rgb8().into_raw();
    for i in 0..width {
        for j in 0..height {
            let index = (j * width + i) * 3;
            let r = pixels[index];
            let g = pixels[index + 1];
            let b = pixels[index + 2];
            pixels[index] = b;
            pixels[index + 1] = g;
            pixels[index + 2] = r;
        }
    }
    ImageWrapper {
        width,
        height,
        channels: 3,
        pixels,
    }
}
fn to_bgr_image(image: &ImageWrapper) -> BGRImage<'_> {
    BGRImage::from_wrapped_pixels(WrappedPixels {
        width: image.width,
        height: image.height,
        channels: image.channels,
        pixels: &image.pixels,
    })
}
#[test]
fn test_locator() {
    for i in 1..=2 {
        let image = ImageReader::open(format!("tests/locator/{i}.png"))
            .unwrap()
            .decode()
            .unwrap();
        let image = bgr_wrapper(&image);
        let image = to_bgr_image(&image);
        let result = locate(&image);
        println!("{result:?}");
    }
}

fn gray_wrapper(image: &DynamicImage) -> ImageWrapper {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let pixels = image.to_luma8().into_raw();
    ImageWrapper {
        width,
        height,
        channels: 1,
        pixels,
    }
}
fn to_character_image(image: &ImageWrapper) -> CharacterImage {
    CharacterImage::from_wrapped_pixels(WrappedPixels {
        width: image.width,
        height: image.height,
        channels: image.channels,
        pixels: &image.pixels,
    })
}

#[test]
fn test_recognize_enemy() {
    let mut images = Vec::new();
    let mut names = Vec::new();
    walkdir::WalkDir::new("tests/recognize_enemy")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|e| {
            let image = ImageReader::open(e.path()).unwrap().decode().unwrap();
            let image = gray_wrapper(&image);
            let image = to_character_image(&image);
            images.push(image);
            names.push(e.file_name().to_string_lossy().to_string());
        });
    println!(
        "Loaded {} templates",
        recognize_enemy::templates::TEMPLATES.len()
    );
    for (i, template) in recognize_enemy::templates::TEMPLATES.iter().enumerate() {
        println!("Template {i}: {}", template.ship_type.as_chinese());
    }
    let result = recognize_enemy::recognize_enemy(&images);

    //match result with names
    for (r, n) in result.iter().zip(names.iter()) {
        println!("{} {} - {n}", r.as_chinese(), r.as_english());
    }
}
