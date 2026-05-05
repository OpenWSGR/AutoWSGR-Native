pub mod character_image;
pub mod templates;

use character_image::{CharacterImage, MatchMethod, MatchResult};
use templates::{TEMPLATES, Template};
use vessel_type::VesselType;

pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 16;
const ENDWITH_巡: &[&str; 6] = &["CA", "CL", "CAV", "CLT", "CBG", "BC"];
const ENDWITH_母: &[&str; 3] = &["CV", "AV", "CVL"];
const STARTWITH_战: &[&str; 2] = &["BB", "BC"];
const STARTWITH_轻: &[&str; 2] = &["CL", "CVL"];

pub fn recognize_enemy(images: &[CharacterImage]) -> Vec<VesselType> {
    let templates = &*TEMPLATES;
    let mut result = Vec::with_capacity(images.len());

    for image in images {
        // pure-constant images are explicit NO
        if matches!(image, CharacterImage::Const) {
            result.push(VesselType::NO);
            continue;
        }

        let mut best: Option<(&Template, MatchResult)> = None;
        for current in templates.iter() {
            let category = match best {
                Some((best_template, _)) => check_category(current, best_template),
                None => Category::None,
            };
            let method = match category {
                Category::Endwith巡 | Category::Endwith母 => MatchMethod::First,
                Category::Startwith战 | Category::Startwith轻 => MatchMethod::Last,
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
            _ => result.push(VesselType::NO),
        }
    }

    result
}

#[derive(Debug)]
enum Category {
    Endwith巡,
    Endwith母,
    Startwith战,
    Startwith轻,
    None,
}
fn check_category(a: &Template, b: &Template) -> Category {
    if ENDWITH_巡.contains(&a.ship_type.as_english())
        && ENDWITH_巡.contains(&b.ship_type.as_english())
    {
        return Category::Endwith巡;
    }
    if ENDWITH_母.contains(&a.ship_type.as_english())
        && ENDWITH_母.contains(&b.ship_type.as_english())
    {
        return Category::Endwith母;
    }
    if STARTWITH_战.contains(&a.ship_type.as_english())
        && STARTWITH_战.contains(&b.ship_type.as_english())
    {
        return Category::Startwith战;
    }
    if STARTWITH_轻.contains(&a.ship_type.as_english())
        && STARTWITH_轻.contains(&b.ship_type.as_english())
    {
        return Category::Startwith轻;
    }
    Category::None
}
