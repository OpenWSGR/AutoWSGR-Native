pub mod image;
pub mod locator;
pub mod recognize_enemy;
pub mod recognize_map;
#[cfg(test)]
mod test;

pub struct WrappedPixels<'i> {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub pixels: &'i [u8],
}
