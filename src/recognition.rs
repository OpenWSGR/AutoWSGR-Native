//! 图像识别后端：在游戏截图中定位舰名、识别敌方舰种与地图节点。
//!
//! 各子模块对外提供纯 Rust 实现，不依赖 pyo3；Python 绑定见 `python_bindings`。

pub mod image;
pub mod locator;
pub mod recognize_enemy;
pub mod recognize_map;
#[cfg(test)]
mod test;

/// 借用外部像素缓冲区的通用包装：行优先存储、各通道交错。
pub struct WrappedPixels<'i> {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub pixels: &'i [u8],
}
