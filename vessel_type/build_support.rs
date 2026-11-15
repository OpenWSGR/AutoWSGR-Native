//! Pure generation rules shared by the build script and its offline tests.

use std::collections::HashMap;

/// Normalize explicitly supported Latin lookalikes, without transliterating words.
///
/// ASCII spelling and case remain unchanged. This is a deliberately bounded
/// character map, not a Unicode confusable skeleton: a skeleton may lowercase
/// uppercase I or merge digits and letters, which would change existing codes.
pub fn normalize_abbreviation(input: &str) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    for original in input.chars() {
        let ch = match original {
            '\u{ff01}'..='\u{ff5e}' => char::from_u32(original as u32 - 0xfee0).unwrap(),
            // Cyrillic and Greek capitals with an ASCII Latin counterpart.
            'А' | 'Α' => 'A',
            'В' | 'Β' => 'B',
            'С' | 'Ϲ' => 'C',
            'Е' | 'Ε' => 'E',
            'Н' | 'Η' => 'H',
            'І' | 'Ι' => 'I',
            'Ј' => 'J',
            'К' | 'Κ' => 'K',
            'М' | 'Μ' => 'M',
            'Ν' => 'N',
            'О' | 'Ο' => 'O',
            'Р' | 'Ρ' => 'P',
            'Ѕ' => 'S',
            'Т' | 'Τ' => 'T',
            'Х' | 'Χ' => 'X',
            'У' | 'Υ' => 'Y',
            'Ζ' => 'Z',
            // Lowercase lookalikes remain lowercase.
            'а' => 'a',
            'с' | 'ϲ' => 'c',
            'е' => 'e',
            'һ' => 'h',
            'і' => 'i',
            'ј' | 'ϳ' => 'j',
            'о' | 'ο' => 'o',
            'р' | 'ρ' => 'p',
            'ѕ' => 's',
            'х' => 'x',
            'у' => 'y',
            ch if ch.is_ascii() => ch,
            _ => {
                return Err(format!(
                    "缩写 {input:?} 包含未支持字符 {original:?} (U+{:04X})",
                    original as u32
                ));
            }
        };
        output.push(ch);
    }
    if output.is_empty()
        || (output != "?"
            && !output
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
    {
        return Err(format!("缩写 {input:?} 归一化后不是合法 ASCII 代号"));
    }
    Ok(output)
}

/// Check collisions after normalization, before any generated file is replaced.
pub fn normalize_unique<'a>(
    inputs: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<String>, String> {
    let mut seen = HashMap::new();
    let mut output = Vec::new();
    for raw in inputs {
        let normalized = normalize_abbreviation(raw)?;
        if let Some(previous) = seen.insert(normalized.clone(), raw) {
            return Err(format!(
                "缩写归一化重名：{previous:?} 与 {raw:?} 都映射为 {normalized:?}"
            ));
        }
        output.push(normalized);
    }
    Ok(output)
}

/// Stable source fingerprint for cache invalidation; not a security checksum.
///
/// `\r\n` is normalized to `\n` first: `include_bytes!` captures raw disk bytes,
/// and with `core.autocrlf=true` on Windows a fresh checkout materializes the
/// sources as CRLF while the committed generators were hashed as LF — without
/// normalization the signature would flip between platforms and spuriously
/// invalidate committed generated files.
pub fn source_signature(sources: &[&[u8]]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for source in sources {
        let normalized: Vec<u8> = source
            .iter()
            .copied()
            .filter(|&b| b != b'\r')
            .collect();
        for byte in (normalized.len() as u64)
            .to_le_bytes()
            .iter()
            .chain(normalized.iter())
        {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

/// A matching wiki revision alone is insufficient after generation rules change.
pub fn generated_matches(
    generated: &str,
    comment_prefix: &str,
    timestamp: &str,
    signature: &str,
) -> bool {
    let timestamp_line = format!("{comment_prefix} Page-Last-Modified: {timestamp}");
    let signature_line = format!("{comment_prefix} Generator-Signature: {signature}");
    generated.lines().any(|line| line == timestamp_line)
        && generated.lines().any(|line| line == signature_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_characters_across_scripts_and_widths() {
        assert_eq!(normalize_abbreviation("КР").unwrap(), "KP");
        assert_eq!(
            normalize_abbreviation("АВCΕНІЈΚΜΝОΡЅТΧΥΖ").unwrap(),
            "ABCEHIJKMNOPSTXYZ"
        );
        assert_eq!(
            normalize_abbreviation("асеһіјоρѕху").unwrap(),
            "acehijopsxy"
        );
        assert_eq!(normalize_abbreviation("ＢＢＧ_２").unwrap(), "BBG_2");
        assert!(normalize_abbreviation("ＣVЛ").is_err());
        assert_eq!(normalize_abbreviation("Еlitе").unwrap(), "Elite");
    }

    #[test]
    fn preserves_ascii_case_and_the_question_mark_sentinel() {
        for text in ["KP", "CVL", "Elite", "Airfield", "NO", "?"] {
            assert_eq!(normalize_abbreviation(text).unwrap(), text);
        }
        assert_eq!(normalize_abbreviation("？").unwrap(), "?");
    }

    #[test]
    fn rejects_unknown_unicode_and_invalid_codes() {
        for text in ["", "Ж", "λ", "é", "航母", "A B", "B/G", "BB?"] {
            assert!(normalize_abbreviation(text).is_err(), "accepted {text:?}");
        }
    }

    #[test]
    fn rejects_collisions_created_by_normalization() {
        assert!(normalize_unique(["KP", "КР"]).is_err());
        assert!(normalize_unique(["BB", "ＢＢ"]).is_err());
        assert!(normalize_unique(["CV", "CV"]).is_err());
        assert_eq!(
            normalize_unique(["Еlitе", "ΚΡ", "?"]).unwrap(),
            ["Elite", "KP", "?"]
        );
    }

    #[test]
    fn signature_is_line_ending_insensitive() {
        assert_eq!(
            source_signature(&[b"fn a() {\r\n    0\r\n}\r\n"]),
            source_signature(&[b"fn a() {\n    0\n}\n"])
        );
    }

    #[test]
    fn unchanged_page_is_invalidated_by_changed_generator_rules() {
        let previous = source_signature(&[b"build v1", b"rules v1"]);
        let changed_rules = source_signature(&[b"build v1", b"rules v2"]);
        let changed_build = source_signature(&[b"build v2", b"rules v1"]);
        let generated =
            format!("// Page-Last-Modified: same-page\n// Generator-Signature: {previous}\n");
        assert!(generated_matches(&generated, "//", "same-page", &previous));
        assert!(!generated_matches(
            &generated,
            "//",
            "same-page",
            &changed_rules
        ));
        assert!(!generated_matches(
            &generated,
            "//",
            "same-page",
            &changed_build
        ));
        assert!(!generated_matches(&generated, "//", "new-page", &previous));
        assert!(!generated_matches(
            "// Page-Last-Modified: same-page\n",
            "//",
            "same-page",
            &previous
        ));
        assert_ne!(
            source_signature(&[b"ab", b"c"]),
            source_signature(&[b"a", b"bc"])
        );
    }
}
