//! 待识别的字符图像及其模板匹配。
//!
//! [`CharacterImage`] 将 16×32 灰度字模预处理为对比度增强后归一化的像素矩阵，
//! 并提供基于归一化互相关（NCC）的 [`CharacterImage::calc_image_difference`]，
//! 供 [`super`] 选择最匹配的舰种模板。

use crate::recognition::WrappedPixels;

use super::{HEIGHT, WIDTH, templates::Template};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
/// 模板匹配时只比较字符图像的哪一段列区间，用于对同字首/同字尾舰种消歧。
pub enum MatchMethod {
    /// 仅比较前半段（差异在首字，如“重巡/轻巡”）。
    First,
    /// 仅比较后半段（差异在末字，如“战列/战巡”）。
    Last,
    /// 比较整段。
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// 单个模板的匹配结果，用于在候选模板间择优。
///
/// 偏序由小到大为：`Score` < `Rejected` < `Uncertain`，
/// 越小越优先被 [`super`] 选中（`Score` 值越小越相似）。
pub enum MatchResult {
    /// 与模板的距离（0 最相似）。
    Score(f64),
    /// 因亮度差异过大直接排除。
    Rejected,
    /// 无法判定（亮度为 0 或分母为 0 等）。
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
/// 归一化后的字符图像，用于与舰种模板做 NCC 匹配。
///
/// [`Const`](CharacterImage::Const) 表示整图灰度一致（无笔画的空白/纯色块），
/// 无需匹配、直接判为 `NO`；[`Image`](CharacterImage::Image) 携带预处理后的像素与亮度。
pub enum CharacterImage {
    /// 灰度均匀的图像（min == max），视为无字符。
    Const,
    /// 含笔画、已完成预处理与归一化的图像。
    Image {
        pixels: [[f64; WIDTH]; HEIGHT],
        brightness: f64,
    },
}

impl CharacterImage {
    /// 由单通道灰度像素构造字符图像：先按 min/max 判定是否为空白，
    /// 再做对比度增强并归一化（详见函数内注释）。
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

    /// 计算本图像与模板在指定列区间上的匹配距离。
    ///
    /// 先按亮度比预筛选（比值 ≥3 判 `Rejected`、亮度为 0 判 `Uncertain`），
    /// 再在 `method` 限定的列区间内对 9 个微位移（dx、dy ∈ {-1,0,1}）取最佳 NCC，
    /// 距离 = 1 − NCC（0 最相似）；任一位移分母为 0 时整体判 `Uncertain`。
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
