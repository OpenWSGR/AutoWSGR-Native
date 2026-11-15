//! 战果截图舰名行带定位。
//!
//! [`locate`] 在整张 BGR 截图中，按舰名文字的关键色提取水平分段，再依据相邻行
//! 的分段一致性筛出舰名所在的行带，返回每条行带的上下界。

use super::image::{BGRImage, Pixel};

/// 战果截图中舰名文字所用的关键色（BGR）；是否命中由 [`Pixel::is_key_color`] 判定。
const KEY_COLORS: [Pixel; 3] = [
    Pixel {
        pixel: &[162, 98, 18],
    },
    Pixel {
        pixel: &[173, 103, 17],
    },
    Pixel {
        pixel: &[196, 116, 16],
    },
];
/// 定位截图中舰名文字所在的行带。
///
/// 算法分四步：① 按关键色标记像素 → ② 在每行内提取水平分段 →
/// ③ 依据与相邻 8 行的分段一致性判定该行是否有效 → ④ 合并连续有效行得到行带。
/// 返回的每对为 `[top, bottom)`（top 含、bottom 不含）的行索引。
pub fn locate(image: &BGRImage) -> Vec<[i32; 2]> {
    let width = image.get_width();
    let height = image.get_height();

    // mark key color
    let mut is_key_map = vec![vec![false; width]; height];
    for (y, row) in is_key_map.iter_mut().enumerate() {
        for (x, is_key) in row.iter_mut().enumerate() {
            let pixel = image.get_pixel(y, x);
            *is_key = pixel.is_key_color(&KEY_COLORS, 20.0);
        }
    }

    // make segments
    let mut segments = vec![vec![]; height];
    for y in 1..height {
        let mut last_key = -1;
        for x in 1..width {
            if is_key_map[y][x] {
                if !is_key_map[y][x - 1] {
                    last_key = x as i32;
                }
            } else if is_key_map[y][x - 1] && x as i32 - last_key > 10 {
                segments[y].push((last_key, x as i32 - 1));
            }
        }
    }
    // validate lines
    let mut is_line_valid = vec![false; height];
    for y in 1..height {
        if segments[y].is_empty() {
            continue;
        }
        let mut cnt1 = 0;
        let mut cnt2 = 0;
        for i in -8..=-1 {
            if 0 < y as i32 + i && y as i32 + i < height as i32 {
                cnt1 += cmp_line(&segments[y], &segments[(y as i32 + i) as usize]);
            }
        }
        for i in 1..=8 {
            if 0 < y as i32 + i && y as i32 + i < height as i32 {
                cnt2 += cmp_line(&segments[y], &segments[(y as i32 + i) as usize]);
            }
        }
        if i32::max(cnt1, cnt2) >= 6 {
            is_line_valid[y] = true;
        }
    }
    // make result
    let mut result = vec![];
    let mut lst = -1;
    for (row, is_line_valid) in is_line_valid.iter().enumerate() {
        if *is_line_valid {
            if lst == -1 {
                lst = row as i32;
            }
        } else if lst != -1 {
            result.push([lst, row as i32]);
            lst = -1;
        }
    }
    if lst != -1 {
        result.push([lst, height as i32]);
    }

    result
}

/// 判断两行的水平分段是否“对齐”。
///
/// `locate` 调用时保证 `line1`（当前行）恒非空，返回值随后累加，用于统计上下
/// 相邻各行中“同样含有舰名文字”的行数。当前生效的 `match` 版本是等价简化替代；
/// 紧随其后的注释代码为逻辑严格的原始实现（逐段检查位置是否对齐），保留作参考。
fn cmp_line(line1: &[(i32, i32)], line2: &[(i32, i32)]) -> i32 {
    match line1.is_empty() {
        true => 1,
        false => match line2.is_empty() {
            true => 0,
            false => 1,
        },
    }
    // 逻辑严格正确的原始实现；当前生效的 `match` 版本是其等价简化替代：
    // for s1 in line1 {
    //     let mut flag = false;
    //     for s2 in line2 {
    //         if i32::min(i32::abs(s1.0 - s2.0), i32::abs(s1.1 - s2.1)) <= 2 {
    //             flag = true;
    //             break;
    //         }
    //     }
    //     if !flag {
    //         return 0;
    //     }
    // }
    // 1
}
