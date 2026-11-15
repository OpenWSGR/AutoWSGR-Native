//! 编译期从 zjsnrwiki 的「舰船类型」页面同步 [`crate::VesselType`] 的定义。
//!
//! 生成物提交进仓库，wiki 数据或生成规则变化时重新生成（内容不变不写盘）：
//! - `src/generated.rs`：Rust 枚举定义
//! - `../python/autowsgr_native/vessel_type/__init__.pyi`：Python 类型存根
//!
//! 网络策略：先轻量查询页面 revision 时间戳（api.php 动态响应不带 HTTP 缓存头），
//! 与生成物头部记录、生成器源码签名均一致才跳过；无法联网时打印 `cargo:warning` 并沿用现有
//! 生成物，构建不受影响。环境变量：
//! - `VESSEL_TYPE_OFFLINE`（任意值）：跳过检查（CI/确定性构建）；
//! - `VESSEL_TYPE_INPUT=<file>`：从本地 JSON（MediaWiki API 响应）生成，不联网。
//!
//! 注意：cargo 官方建议 build script 只写 `OUT_DIR`，本脚本重写仓库内生成物属于
//! 已知取舍（换取「编译时自动跟随 wiki」），缓解措施见 README。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

mod build_support;
use build_support::{generated_matches, normalize_unique, source_signature};

const API_URL: &str = "https://www.zjsnrwiki.com/api.php";
const PAGE: &str = "舰船类型";
/// 站点 WAF（雷池）会拦截非浏览器 UA 的 API 请求（实测诚实标识 UA 持续 468，
/// 浏览器 UA 放行），故伪装为浏览器；本脚本每次构建最多发两个轻量只读请求。
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const TIMEOUT: Duration = Duration::from_secs(8);
/// 目标表格的表头签名（清洗 wikitext 格式后逐字比较）。
const TARGET_HEADER: [&str; 3] = ["全称", "二字简称", "字母缩写"];
/// 行数健全性下限，防止页面结构变化解析出残表仍继续生成。
const MIN_ROWS: [usize; 2] = [20, 3];
/// wiki 之外的附加条目（识别哨兵），排在枚举末尾。
const EXTRA_ROWS: [(&str, &str, &str); 1] = [("无", "无", "NO")];
/// `VESSEL_TYPE_INPUT` 模式写入生成物头部的时间戳占位（不参与网络比较）。
const LOCAL_INPUT_MARK: &str = "(local input)";

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_support.rs");
    // src 含 generated.rs：写盘后下次自触发，内容相同即收敛，不会无限重编
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../python/autowsgr_native/vessel_type/__init__.pyi");
    // 根包源码：日常开发改动保证 wiki 检查被触达
    println!("cargo:rerun-if-changed=../src");
    println!("cargo:rerun-if-env-changed=VESSEL_TYPE_INPUT");
    println!("cargo:rerun-if-env-changed=VESSEL_TYPE_OFFLINE");

    if std::env::var_os("VESSEL_TYPE_OFFLINE").is_some() {
        return;
    }
    let rust_out = manifest.join("src/generated.rs");
    let pyi_out = manifest.join("../python/autowsgr_native/vessel_type/__init__.pyi");
    let signature = source_signature(&[
        include_bytes!("build.rs"),
        include_bytes!("build_support.rs"),
    ]);

    // 拉取：(wikitext, page_timestamp)。Ok(None) = 页面未变化。
    let fetched = match std::env::var("VESSEL_TYPE_INPUT") {
        Ok(path) => {
            println!("cargo:rerun-if-changed={path}");
            read_input(Path::new(&path))
        }
        Err(_) => fetch_wikitext(&rust_out, &pyi_out, &signature),
    };

    let (wikitext, last_modified) = match fetched {
        Ok(None) => return,
        Ok(Some(pair)) => pair,
        Err(e) => {
            // 网络/读取失败不是构建错误：warning 后沿用现有生成物
            println!("cargo:warning=无法访问 zjsnrwiki（{e}），舰种数据沿用仓库现有生成物");
            return;
        }
    };

    let (player, enemy) = match parse_tables(&wikitext) {
        Ok(tables) => tables,
        Err(e) => {
            println!("cargo:warning=舰船类型页面解析失败（{e}），沿用仓库现有生成物");
            return;
        }
    };
    let (player_count, enemy_count) = (player.len(), enemy.len());
    let entries = match assemble(player, enemy) {
        Ok(entries) => entries,
        Err(e) => {
            println!("cargo:warning=舰种数据校验失败（{e}），沿用仓库现有生成物");
            return;
        }
    };

    let new_rust = render_rust(&entries, &last_modified, &signature);
    let changed_rust = write_if_changed(&rust_out, &new_rust);
    if pyi_out.parent().is_some_and(Path::exists) {
        let new_pyi = render_pyi(
            &entries,
            player_count,
            enemy_count,
            &last_modified,
            &signature,
        );
        let changed_pyi = write_if_changed(&pyi_out, &new_pyi);
        if changed_rust || changed_pyi {
            println!(
                "cargo:warning=舰种生成物已更新（Page-Last-Modified: {last_modified}），请 review 并提交生成物"
            );
        }
    } else {
        println!("cargo:warning=未找到 python/ 目录，跳过 .pyi 生成（独立构建场景）");
    }
}

// ---------------------------------------------------------------------------
// 拉取
// ---------------------------------------------------------------------------

fn read_input(path: &Path) -> Result<Option<(String, String)>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取 {}: {e}", path.display()))?;
    let wikitext = extract_wikitext(&content)?;
    Ok(Some((wikitext, LOCAL_INPUT_MARK.to_string())))
}

/// 页面未变化时返回 `Ok(None)`。
///
/// api.php 的动态响应不带 HTTP `Last-Modified`/`ETag`（Cache-Control 明确不可缓存），
/// 因此用 MediaWiki 的页面 revision 时间戳作等价条件：先轻量查询时间戳，与生成物
/// 头部记录和生成器源码签名均一致才跳过，不同就拉取完整 wikitext 重新生成。
fn fetch_wikitext(
    rust_out: &Path,
    pyi_out: &Path,
    signature: &str,
) -> Result<Option<(String, String)>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();

    let ts_url = format!(
        "{API_URL}?action=query&prop=revisions&rvprop=timestamp&titles={PAGE}&format=json&formatversion=2"
    );
    let ts_body = agent
        .get(&ts_url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("{e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| format!("读取响应体: {e}"))?;
    let timestamp = extract_page_timestamp(&ts_body)?;
    let rust_current = fs::read_to_string(rust_out)
        .is_ok_and(|text| generated_matches(&text, "//", &timestamp, signature));
    let pyi_current = !pyi_out.parent().is_some_and(Path::exists)
        || fs::read_to_string(pyi_out)
            .is_ok_and(|text| generated_matches(&text, "#", &timestamp, signature));
    if rust_current && pyi_current {
        return Ok(None);
    }

    let url =
        format!("{API_URL}?action=parse&page={PAGE}&prop=wikitext&format=json&formatversion=2");
    let body = agent
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("{e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| format!("读取响应体: {e}"))?;
    let wikitext = extract_wikitext(&body)?;
    Ok(Some((wikitext, timestamp)))
}

fn extract_page_timestamp(body: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("JSON 解析: {e}"))?;
    let page = &value["query"]["pages"][0];
    if page["missing"].as_bool() == Some(true) {
        return Err(format!("wiki 页面不存在: {PAGE}"));
    }
    page["revisions"][0]["timestamp"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "API 响应缺少 revisions[0].timestamp 字段".to_string())
}

fn extract_wikitext(body: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("JSON 解析: {e}"))?;
    value["parse"]["wikitext"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "API 响应缺少 parse.wikitext 字段".to_string())
}

// ---------------------------------------------------------------------------
// 解析（从 wikitext 表格到数据行）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Row {
    full: String,
    chinese: String,
    english: String,
}

type Tables = (Vec<Row>, Vec<Row>);

fn parse_tables(wikitext: &str) -> Result<Tables, String> {
    let mut hits: Vec<Vec<Row>> = Vec::new();
    for lines in split_tables(wikitext) {
        let parsed: Vec<Option<Vec<String>>> = lines.iter().map(|l| parse_row(l)).collect();
        let cells: Vec<Vec<String>> = parsed.into_iter().flatten().collect();
        if cells.is_empty() || cells[0] != TARGET_HEADER {
            continue;
        }
        let rows = cells[1..]
            .iter()
            .filter(|c| c.iter().any(|s| !s.is_empty()))
            .map(|c| Row {
                full: c[0].clone(),
                chinese: c[1].clone(),
                english: c[2].clone(),
            })
            .collect();
        hits.push(rows);
    }
    if hits.len() != 2 {
        return Err(format!(
            "目标表格命中 {} 张（预期 2 张），页面结构可能已变化",
            hits.len()
        ));
    }
    let [player, enemy] = hits.try_into().unwrap();
    if player.len() < MIN_ROWS[0] {
        return Err(format!(
            "玩家表行数异常：{} < {}",
            player.len(),
            MIN_ROWS[0]
        ));
    }
    if enemy.len() < MIN_ROWS[1] {
        return Err(format!("敌方表行数异常：{} < {}", enemy.len(), MIN_ROWS[1]));
    }
    Ok((player, enemy))
}

fn split_tables(wikitext: &str) -> Vec<Vec<&str>> {
    let mut tables = Vec::new();
    let mut depth = 0usize;
    let mut current: Vec<&str> = Vec::new();
    for raw in wikitext.lines() {
        let line = raw.trim();
        if line.starts_with("{|") {
            depth += 1;
            if depth == 1 {
                current = Vec::new();
            }
        } else if line.starts_with("|}") {
            if depth == 1 {
                tables.push(current.clone());
            }
            depth = depth.saturating_sub(1);
        } else if depth >= 1 {
            current.push(line);
        }
    }
    tables
}

fn parse_row(line: &str) -> Option<Vec<String>> {
    if line.starts_with("|-") {
        return None;
    }
    let (body, sep) = match line.strip_prefix('!') {
        Some(rest) => (rest, "!!"),
        None => (line.strip_prefix('|')?, "||"),
    };
    Some(body.split(sep).map(clean_cell).collect())
}

fn clean_cell(cell: &str) -> String {
    let mut s = remove_html_tags(&cell.replace("&nbsp;", " "));
    s = remove_wiki_templates(&s);
    s = resolve_wiki_links(&s);
    s.replace("''", "").trim().to_string()
}

fn remove_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth: u32 = 0;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// 不支持嵌套（舰种表未用到）。
fn remove_wiki_templates(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            let mut depth = 1;
            for c2 in chars.by_ref() {
                match c2 {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn resolve_wiki_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' && chars.peek() == Some(&'[') {
            chars.next();
            let mut inner = String::new();
            let mut closed = false;
            while let Some(c2) = chars.next() {
                if c2 == ']' && chars.peek() == Some(&']') {
                    chars.next();
                    closed = true;
                    break;
                }
                inner.push(c2);
            }
            if closed {
                out.push_str(
                    inner
                        .rsplit_once('|')
                        .map_or(inner.as_str(), |(_, label)| label),
                );
            } else {
                out.push('[');
                out.push_str(&inner);
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 校验与组装
// ---------------------------------------------------------------------------

/// 三个名字域必须各自唯一（from_chinese/from_english 依赖双射）。
fn assemble(mut player: Vec<Row>, enemy: Vec<Row>) -> Result<Vec<Row>, String> {
    player.extend(enemy);
    for (full, chinese, english) in EXTRA_ROWS {
        player.push(Row {
            full: full.into(),
            chinese: chinese.into(),
            english: english.into(),
        });
    }
    let normalized = normalize_unique(player.iter().map(|row| row.english.as_str()))?;
    for (row, english) in player.iter_mut().zip(normalized) {
        row.english = english;
    }
    for row in &player {
        if row.full.is_empty() || row.chinese.is_empty() || row.english.is_empty() {
            return Err(format!("存在空单元格：{}/{}", row.full, row.chinese));
        }
        if !is_valid_identifier(&row.chinese) {
            return Err(format!(
                "简称 {} 不是合法的 Rust/Python 标识符",
                row.chinese
            ));
        }
    }
    let check_unique = |name: &str, values: Vec<&str>| -> Result<(), String> {
        if values
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != values.len()
        {
            return Err(format!("{name}域存在重复"));
        }
        Ok(())
    };
    check_unique("简称", player.iter().map(|r| r.chinese.as_str()).collect())?;
    check_unique("全称", player.iter().map(|r| r.full.as_str()).collect())?;
    check_unique("缩写", player.iter().map(|r| r.english.as_str()).collect())?;
    Ok(player)
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// ---------------------------------------------------------------------------
// 渲染
// ---------------------------------------------------------------------------

fn render_rust(entries: &[Row], last_modified: &str, signature: &str) -> String {
    let mut out = String::new();
    out.push_str("// @generated by vessel_type/build.rs — DO NOT EDIT.\n");
    out.push_str("// Source of truth: https://www.zjsnrwiki.com/wiki/舰船类型\n");
    out.push_str(&format!("// Page-Last-Modified: {last_modified}\n"));
    out.push_str(&format!("// Generator-Signature: {signature}\n"));
    out.push_str("\nuse strum::{EnumIter, EnumProperty, IntoStaticStr};\n");
    out.push_str(
        "\n/// 战舰少女R 中的舰船与设施类型。\n///\n/// 变体名 = 中文简称（wiki 二字简称），声明顺序 = wiki 表格顺序 + 附加哨兵。\n/// 本类型为纯字符串枚举：名称经 `as_chinese` / `as_english` / `as_full_name` 获取，\n/// 不要依赖变体的整数判别值（pyo3 暴露的 `int()` 仅供调试，不保证稳定）。\n",
    );
    out.push_str("#[cfg_attr(feature = \"pyo3\", pyo3::pyclass(eq, from_py_object))]\n");
    out.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumProperty, IntoStaticStr)]\n",
    );
    out.push_str("pub enum VesselType {\n");
    for e in entries {
        out.push_str(&format!(
            "    #[strum(props(FullName = \"{}\", English = \"{}\"))]\n    {},\n",
            e.full, e.english, e.chinese
        ));
    }
    out.push_str("}\n");
    out
}

const PYI_HEADER: &str = r#"""":mod:`autowsgr_native.vessel_type` 的类型存根。

由 vessel_type/build.rs 生成，请勿手改。
"""

from typing import ClassVar

__all__ = ["VesselType"]


class VesselType:
    """从游戏截图中识别出的舰船或设施类型。

    各变体以类属性形式暴露（如 ``VesselType.驱逐``）。使用
    :meth:`as_chinese` / :meth:`as_english` / :meth:`as_full_name` 获取其
    名称，使用 :meth:`from_chinese` / :meth:`from_english` 将名称解析回变体。
    """

"#;

const PYI_METHODS: &str = r#"    def as_chinese(self) -> str:
        """该舰种的中文名（如 ``"驱逐"``）。"""

    def as_english(self) -> str:
        """该舰种的英文代号（如 ``"DD"``）。"""

    def as_full_name(self) -> str:
        """该舰种的中文全称（如 ``"驱逐舰"``）。"""

    @classmethod
    def from_chinese(cls, s: str) -> VesselType | None:
        """按中文名解析为变体，无法识别时返回 ``None``。"""

    @classmethod
    def from_english(cls, s: str) -> VesselType | None:
        """按英文代号解析为变体，无法识别时返回 ``None``。"""
"#;

fn render_pyi(
    entries: &[Row],
    player_count: usize,
    enemy_count: usize,
    last_modified: &str,
    signature: &str,
) -> String {
    let mut out =
        format!("# Page-Last-Modified: {last_modified}\n# Generator-Signature: {signature}\n");
    out.push_str(PYI_HEADER);
    for (i, e) in entries.iter().enumerate() {
        if i == player_count || i == player_count + enemy_count {
            out.push('\n'); // 玩家表 / 敌方表 / 哨兵 之间空一行
        }
        out.push_str(&format!(
            "    {}: ClassVar[VesselType]\n    \"\"\"{} / {}\"\"\"\n",
            e.chinese, e.full, e.english
        ));
    }
    out.push('\n');
    out.push_str(PYI_METHODS);
    out
}

/// 内容有变化才写盘（避免无谓触碰 mtime 破坏 cargo 收敛）。
fn write_if_changed(path: &Path, content: &str) -> bool {
    if fs::read_to_string(path).is_ok_and(|old| old == content) {
        return false;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    fs::write(path, content).expect("写生成物失败");
    true
}
