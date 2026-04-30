use crate::interface::WrappedPixels;

use super::{HEIGHT, WIDTH, templates::Template};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchMethod {
    First,
    Last,
    All,
}

#[derive(Debug, Clone)]
pub struct CharacterImage {
    pub width: usize,
    pub height: usize,
    pub pixels: [[f64; WIDTH]; HEIGHT],
    pub brightness: f64,
}
impl CharacterImage {
    pub fn from_wrapped_pixels(wrapped_pixels: WrappedPixels) -> Self {
        let width = wrapped_pixels.width;
        let height = wrapped_pixels.height;
        assert_eq!(wrapped_pixels.channels, 1);
        let pixels = wrapped_pixels.pixels;
        assert_eq!(width, WIDTH);
        assert_eq!(height, HEIGHT);
        let mut sum = 0f64;
        let mut image = CharacterImage {
            width,
            height,
            pixels: [[0f64; WIDTH]; HEIGHT],
            brightness: 0f64,
        };
        for i in 0..height {
            for j in 0..width {
                let pixel = pixels[i * width + j];
                image.pixels[i][j] = pixel as f64;
                sum += pixel as f64;
            }
        }
        image.brightness = sum;
        image.process();
        image
    }
    fn process(&mut self) {
        let mut sum = 0f64;
        for row in &mut self.pixels {
            for pixel in row {
                if *pixel > self.brightness / self.width as f64 / self.height as f64 * 1.2 {
                    *pixel += 10f64;
                }
                if *pixel < self.brightness / self.width as f64 / self.height as f64 * 0.8 {
                    *pixel -= 10f64;
                }
                sum += *pixel;
            }
        }
        for row in &mut self.pixels {
            for pixel in row {
                *pixel /= sum;
            }
        }
    }

    /// Calculate image difference using NCC (Normalized Cross-Correlation).
    /// Returns a "distance" value: 1 - NCC, where lower is more similar.
    /// NCC is inherently robust to brightness/contrast changes and spreads
    /// the influence of individual pixel mismatches across the whole region.
    pub fn calc_image_difference(&self, template: &Template, method: MatchMethod) -> f64 {
        if self.brightness.max(template.image.brightness)
            / self.brightness.min(template.image.brightness)
            >= 3f64
        {
            return f64::MAX;
        }
        let match_start;
        let match_end;
        match method {
            MatchMethod::First => {
                match_start = 0;
                match_end = self.width / 2;
            }
            MatchMethod::Last => {
                match_start = self.width / 2;
                match_end = self.width;
            }
            MatchMethod::All => {
                match_start = 0;
                match_end = self.width;
            }
        }

        let mut best_dist = f64::MAX;
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                // Compute means
                let mut count = 0usize;
                let mut sum_a = 0f64;
                let mut sum_b = 0f64;
                for y in 0..self.height as i32 {
                    for x in match_start as i32..match_end as i32 {
                        let tx = x + dx;
                        let ty = y + dy;
                        if tx < match_end as i32
                            && ty < self.height as i32
                            && tx >= match_start as i32
                            && ty >= 0
                        {
                            sum_a += self.pixels[y as usize][x as usize];
                            sum_b += template.image.pixels[ty as usize][tx as usize];
                            count += 1;
                        }
                    }
                }
                let mean_a = sum_a / count as f64;
                let mean_b = sum_b / count as f64;

                // Compute NCC: Σ(a-mean_a)(b-mean_b) / sqrt(Σ(a-mean_a)² · Σ(b-mean_b)²)
                let mut numerator = 0f64;
                let mut denom_a = 0f64;
                let mut denom_b = 0f64;
                for y in 0..self.height as i32 {
                    for x in match_start as i32..match_end as i32 {
                        let tx = x + dx;
                        let ty = y + dy;
                        if tx < match_end as i32
                            && ty < self.height as i32
                            && tx >= match_start as i32
                            && ty >= 0
                        {
                            let a = self.pixels[y as usize][x as usize] - mean_a;
                            let b = template.image.pixels[ty as usize][tx as usize] - mean_b;
                            numerator += a * b;
                            denom_a += a * a;
                            denom_b += b * b;
                        }
                    }
                }
                let denominator = (denom_a * denom_b).sqrt();
                if denominator > 0.0 {
                    let ncc = numerator / denominator;
                    // Convert similarity [-1, 1] to distance: lower = more similar
                    let dist = 1.0 - ncc;
                    best_dist = best_dist.min(dist);
                }
            }
        }
        best_dist
    }
}
