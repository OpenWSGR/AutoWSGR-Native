use crate::WrappedPixels;

use super::{HEIGHT, WIDTH, templates::Template};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchMethod {
    First,
    Last,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchResult {
    Score(f64),
    Rejected,
    Uncertain,
}

impl PartialOrd for MatchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use MatchResult::*;
        match (self, other) {
            (Score(a), Score(b)) => a.partial_cmp(b),
            (Score(_), Rejected) => Some(Ordering::Less),
            (Score(_), Uncertain) => Some(Ordering::Less),
            (Rejected, Score(_)) => Some(Ordering::Greater),
            (Rejected, Rejected) => Some(Ordering::Equal),
            (Rejected, Uncertain) => Some(Ordering::Less),
            (Uncertain, Uncertain) => Some(Ordering::Equal),
            (Uncertain, _) => Some(Ordering::Greater),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum CharacterImage {
    Const,
    Image {
        pixels: [[f64; WIDTH]; HEIGHT],
        brightness: f64,
    },
}

impl CharacterImage {
    pub fn from_wrapped_pixels(wrapped_pixels: WrappedPixels) -> Self {
        let width = wrapped_pixels.width;
        let height = wrapped_pixels.height;
        assert_eq!(
            wrapped_pixels.channels, 1,
            "expected 1 channel for character image, got {} channels",
            wrapped_pixels.channels
        );
        let pixels_raw = wrapped_pixels.pixels;
        assert_eq!(width, WIDTH, "expected width {}, got {}", WIDTH, width);
        assert_eq!(height, HEIGHT, "expected height {}, got {}", HEIGHT, height);

        // compute min/max and brightness sum
        let mut sum = 0f64;
        let mut min = 255u8;
        let mut max = 0u8;

        let mut pixels = [[0f64; WIDTH]; HEIGHT];
        for i in 0..height {
            for j in 0..width {
                let pixel = pixels_raw[i * width + j];
                min = min.min(pixel);
                max = max.max(pixel);
                let pixel = pixel as f64;
                pixels[i][j] = pixel;
                sum += pixel;
            }
        }
        if min == max {
            return CharacterImage::Const;
        }
        // process: adjust contrast-like offsets and normalize
        let mut proc_sum = 0f64;
        let avg = sum / width as f64 / height as f64;
        for row in &mut pixels {
            for pixel in row {
                if *pixel > avg * 1.2 {
                    *pixel += 10f64;
                }
                if *pixel < avg * 0.8 {
                    *pixel -= 10f64;
                }
                proc_sum += *pixel;
            }
        }
        for row in &mut pixels {
            for pixel in row {
                *pixel /= proc_sum;
            }
        }
        CharacterImage::Image {
            pixels,
            brightness: sum,
        }
    }

    pub fn calc_image_difference(&self, template: &Template, method: MatchMethod) -> MatchResult {
        // calc_image_difference should only be called for Image variants; assert to catch logic errors
        let (pixels_a, brightness_a) = match self {
            CharacterImage::Image { pixels, brightness } => (pixels, *brightness),
            CharacterImage::Const => panic!("calc_image_difference called with Const image"),
        };
        let (pixels_b, brightness_b) = match &template.image {
            CharacterImage::Image { pixels, brightness } => (pixels, *brightness),
            CharacterImage::Const => {
                panic!("calc_image_difference called with Const template image")
            }
        };

        if brightness_a == 0.0 || brightness_b == 0.0 {
            return MatchResult::Uncertain;
        }
        if brightness_a.max(brightness_b) / brightness_a.min(brightness_b) >= 3f64 {
            return MatchResult::Rejected;
        }

        let (match_start, match_end) = match method {
            MatchMethod::First => (0usize, WIDTH / 2),
            MatchMethod::Last => (WIDTH / 2, WIDTH),
            MatchMethod::All => (0usize, WIDTH),
        };

        let mut best_dist = f64::MAX;
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                // Compute means
                let mut count = 0usize;
                let mut sum_a = 0f64;
                let mut sum_b = 0f64;
                for y in 0..HEIGHT as i32 {
                    for x in match_start as i32..match_end as i32 {
                        let tx = x + dx;
                        let ty = y + dy;
                        if tx < match_end as i32
                            && ty < HEIGHT as i32
                            && tx >= match_start as i32
                            && ty >= 0
                        {
                            sum_a += pixels_a[y as usize][x as usize];
                            sum_b += pixels_b[ty as usize][tx as usize];
                            count += 1;
                        }
                    }
                }
                if count == 0 {
                    continue;
                }
                let mean_a = sum_a / count as f64;
                let mean_b = sum_b / count as f64;

                // Compute NCC
                let mut numerator = 0f64;
                let mut denom_a = 0f64;
                let mut denom_b = 0f64;
                for y in 0..HEIGHT as i32 {
                    for x in match_start as i32..match_end as i32 {
                        let tx = x + dx;
                        let ty = y + dy;
                        if tx < match_end as i32
                            && ty < HEIGHT as i32
                            && tx >= match_start as i32
                            && ty >= 0
                        {
                            let a = pixels_a[y as usize][x as usize] - mean_a;
                            let b = pixels_b[ty as usize][tx as usize] - mean_b;
                            numerator += a * b;
                            denom_a += a * a;
                            denom_b += b * b;
                        }
                    }
                }
                let denominator = (denom_a * denom_b).sqrt();
                if denominator > 0.0 {
                    let ncc = numerator / denominator;
                    let dist = 1.0 - ncc;
                    if dist < best_dist {
                        best_dist = dist;
                    }
                } else {
                    return MatchResult::Uncertain;
                }
            }
        }

        if best_dist.is_finite() && best_dist < f64::MAX {
            MatchResult::Score(best_dist)
        } else {
            MatchResult::Rejected
        }
    }
}
