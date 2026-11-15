//! 舰种字符模板：在编译期由 `build.rs` 把 `TemplateData/` 下的 PNG 打包为
//! `templates.bin`，运行时由 [`Template::init_templates`] 解析为 [`Template`] 列表。

use std::sync::LazyLock;

use crate::recognition::WrappedPixels;

use super::CharacterImage;
use vessel_type::VesselType;

/// 所有舰种模板，在首次访问时由 [`Template::init_templates`] 惰性解析。
pub static TEMPLATES: LazyLock<Vec<Template>> = LazyLock::new(Template::init_templates);

/// 一个舰种模板：其 [`VesselType`] 与对应的 [`CharacterImage`] 样本。
pub struct Template {
    /// 该模板对应的舰种。
    pub ship_type: VesselType,
    /// 用于匹配的字符样本图像。
    pub image: CharacterImage,
}

impl Template {
    /// # Layout of templates.bin:
    /// | templates_count: u32 | templates: [Template] |
    /// # Layout of Template:
    /// | name_len: u8 | name: [u8; name_len] (UTF-8 中文名) | height: u8 | width: u8 | pixels: [u8] |
    fn init_templates() -> Vec<Template> {
        let raw = include_bytes!(concat!(env!("OUT_DIR"), "/templates.bin"));
        let mut templates = Vec::new();
        let mut offset = 0;
        let template_count = u32::from_ne_bytes([
            raw[offset],
            raw[offset + 1],
            raw[offset + 2],
            raw[offset + 3],
        ]) as usize;
        offset += size_of::<u32>();
        for _ in 0..template_count {
            let name_len = raw[offset] as usize;
            offset += size_of::<u8>();
            let name = std::str::from_utf8(&raw[offset..offset + name_len]).unwrap();
            let ship_type = VesselType::from_chinese(name)
                .unwrap_or_else(|| panic!("Unknown ship type in templates.bin: {name}"));
            offset += name_len;
            let height = raw[offset] as usize;
            offset += size_of::<u8>();
            let width = raw[offset] as usize;
            offset += size_of::<u8>();
            let pixels = &raw[offset..offset + height * width];
            offset += height * width;
            let pixels = WrappedPixels {
                width,
                height,
                channels: 1,
                pixels,
            };
            let image = CharacterImage::from_wrapped_pixels(pixels);
            templates.push(Template { ship_type, image });
        }
        templates
    }
}
