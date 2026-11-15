//! 敌方舰种识别：对预先切分的字符图块逐一与舰种模板做 NCC 匹配，
//! 取最相似的模板作为舰种；空白图块直接判为 `无`。

pub mod character_image;
pub mod templates;

use character_image::{CharacterImage, MatchMethod, MatchResult};
use templates::{TEMPLATES, Template};
use vessel_type::VesselType;
/// 单个字符图像的宽度（像素列数）。
pub const WIDTH: usize = 32;
/// 单个字符图像的高度（像素行数）。
pub const HEIGHT: usize = 16;

/// 识别一组字符图块各自对应的敌方舰种。
///
/// 对每个图块：空白（[`CharacterImage::Const`]）直接判为 [`VesselType::无`]；
/// 否则在所有模板中选出匹配距离最小者（用 [`check_category`] 对同字首/尾舰种做部分匹配消歧），
/// 仅当最佳结果为 [`MatchResult::Score`] 时采用，否则记为 `无`。
pub fn recognize_enemy(images: &[CharacterImage]) -> Vec<VesselType> {
    let templates = &*TEMPLATES;
    let mut result = Vec::with_capacity(images.len());

    for image in images {
        if matches!(image, CharacterImage::Const) {
            result.push(VesselType::无);
            continue;
        }

        let mut best: Option<(&Template, MatchResult)> = None;
        for current in templates.iter() {
            let category = match best {
                Some((best_template, _)) => check_category(current, best_template),
                None => Category::None,
            };
            let method = match category {
                Category::ShareTail => MatchMethod::First,
                Category::ShareHead => MatchMethod::Last,
                Category::None => MatchMethod::All,
            };
            let dist_current = image.calc_image_difference(current, method);
            match best {
                None => best = Some((current, dist_current)),
                Some((_, best_dist)) => {
                    if dist_current < best_dist {
                        best = Some((current, dist_current));
                    }
                }
            }
        }

        match best {
            Some((template, MatchResult::Score(_))) => result.push(template.ship_type),
            _ => result.push(VesselType::无),
        }
    }

    result
}

#[derive(Debug, PartialEq)]
/// 两个候选舰种是否属于需要部分匹配消歧的同一组（按中文名首/尾字划分）。
enum Category {
    /// 中文名尾字相同（如 重巡/轻巡）→ 差异在首字。
    ShareTail,
    /// 中文名首字相同（如 战列/战巡）→ 差异在尾字。
    ShareHead,
    None,
}

/// 判断两个模板的舰种是否属于同一消歧组，用于决定部分匹配区间。
fn check_category(a: &Template, b: &Template) -> Category {
    category_of(a.ship_type.as_chinese(), b.ship_type.as_chinese())
}

/// 按中文名首/尾字判断两个舰种是否属于同一消歧组。
///
/// 尾字相同优先于首字相同（如 战巡 与 重巡 比首字，而与 战列 比尾字）。
/// 新增舰种无需登记分组，天然按首尾字入组。
fn category_of(a: &str, b: &str) -> Category {
    if a.chars().last() == b.chars().last() {
        Category::ShareTail
    } else if a.chars().next() == b.chars().next() {
        Category::ShareHead
    } else {
        Category::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_of() {
        use Category::{None, ShareHead, ShareTail};
        // 尾字相同 → 比首字
        assert_eq!(category_of("战巡", "重巡"), ShareTail);
        assert_eq!(category_of("航母", "轻母"), ShareTail);
        // 首字相同 → 比尾字
        assert_eq!(category_of("战巡", "战列"), ShareHead);
        assert_eq!(category_of("轻巡", "轻母"), ShareHead);
        // 首尾都不同 → 全字匹配
        assert_eq!(category_of("重炮", "驱逐"), None);
    }
}
