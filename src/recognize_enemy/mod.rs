pub mod character_image;
pub mod templates;

use character_image::{CharacterImage, MatchMethod};
use templates::{TEMPLATES, Template};

pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 16;
const ENDWITH_巡: &[&str; 6] = &["CA", "CL", "CAV", "CLT", "CBG", "BC"];
const ENDWITH_母: &[&str; 3] = &["CV", "AV", "CVL"];
const STARTWITH_战: &[&str; 2] = &["BB", "BC"];
const STARTWITH_轻: &[&str; 2] = &["CL", "CVL"];

pub fn recognize_enemy(images: &[CharacterImage]) -> String {
    let templates = &*TEMPLATES;
    let mut string = String::new();

    for image in images {
        let mut best = &templates[0];
        for current in templates.iter().skip(1) {
            let category = check_category(best, current);
            let method = match category {
                Category::Endwith巡 | Category::Endwith母 => MatchMethod::First,
                Category::Startwith战 | Category::Startwith轻 => MatchMethod::Last,
                Category::None => MatchMethod::All,
            };
            let dist_current = image.calc_image_difference(current, method);
            let dist_best = image.calc_image_difference(best, method);
            if dist_current < dist_best {
                best = current;
            }
        }
        string.push_str(best.ship_type.as_ref());
        string.push(' ');
    }

    string
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
    if ENDWITH_巡.contains(&a.ship_type.as_ref()) && ENDWITH_巡.contains(&b.ship_type.as_ref()) {
        return Category::Endwith巡;
    }
    if ENDWITH_母.contains(&a.ship_type.as_ref()) && ENDWITH_母.contains(&b.ship_type.as_ref()) {
        return Category::Endwith母;
    }
    if STARTWITH_战.contains(&a.ship_type.as_ref()) && STARTWITH_战.contains(&b.ship_type.as_ref())
    {
        return Category::Startwith战;
    }
    if STARTWITH_轻.contains(&a.ship_type.as_ref()) && STARTWITH_轻.contains(&b.ship_type.as_ref())
    {
        return Category::Startwith轻;
    }
    Category::None
}
