//! 像素与 BGR 图像的轻量抽象。
//!
//! 提供按通道切片引用的 [`Pixel`] 与按行优先存储的 [`BGRImage`]，
//! 支持基于欧氏距离的关键色匹配，供各识别模块共用。

use super::WrappedPixels;

/// 单个像素，按通道顺序存储其分量（如 BGR 的 3 个字节）。
pub struct Pixel<'i> {
    pub pixel: &'i [u8],
}
impl<'i> Pixel<'i> {
    /// 判断本像素与任一关键色的欧氏距离是否小于 `threshold`。
    pub fn is_key_color(&self, key_colors: &[Pixel], threshold: f64) -> bool {
        for key_color in key_colors {
            let distance = self.calc_distance(key_color);
            if distance < threshold {
                return true;
            }
        }
        false
    }
    /// 计算两像素各通道差值平方和的平方根（即欧氏距离）。
    fn calc_distance(&self, other: &Pixel) -> f64 {
        let mut distance = 0f64;
        for (&a, &b) in self.pixel.iter().zip(other.pixel.iter()) {
            let diff = a as f64 - b as f64;
            distance += diff * diff;
        }
        distance.sqrt()
    }
}
/// 行优先存储、按 BGR 通道排列的图像视图。
///
/// 借用外部像素缓冲区，零拷贝地按 `(y, x)` 取 [`Pixel`]。
pub struct BGRImage<'i> {
    width: usize,
    height: usize,
    channels: usize,
    pixels: &'i [u8],
}
impl<'i> BGRImage<'i> {
    /// 由 [`WrappedPixels`] 构造，要求通道数恰好为 3。
    pub fn from_wrapped_pixels(wrapped_pixels: WrappedPixels<'i>) -> Self {
        assert_eq!(
            wrapped_pixels.channels, 3,
            "expected 3 channels for BGR image, got {} channels",
            wrapped_pixels.channels
        );
        Self {
            width: wrapped_pixels.width,
            height: wrapped_pixels.height,
            channels: wrapped_pixels.channels,
            pixels: wrapped_pixels.pixels,
        }
    }
    /// 图像宽度（像素列数）。
    pub fn get_width(&self) -> usize {
        self.width
    }
    /// 图像高度（像素行数）。
    pub fn get_height(&self) -> usize {
        self.height
    }
    /// 取第 `y` 行、第 `x` 列像素。
    pub fn get_pixel(&self, y: usize, x: usize) -> Pixel<'_> {
        let index = (y * self.width + x) * self.channels;
        let pixel = &self.pixels[index..index + self.channels];
        Pixel { pixel }
    }
}
