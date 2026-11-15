//! 舰种（及港口设施）类型枚举。
//!
//! 枚举定义由 `vessel_type/build.rs` 从 zjsnrwiki 生成（见 [`generated`]），
//! 变体名为中文简称（wiki 二字简称）。每个变体携带中文全称（[`VesselType::as_full_name`]）、
//! 中文简称（[`VesselType::as_chinese`]）与英文代号（[`VesselType::as_english`]），
//! 英文代号在生成时归一化为 ASCII（保留大小写及 `?` 哨兵），并可双向解析。
//! 开启 `pyo3` feature 时作为 Python 类暴露。

mod generated;

pub use generated::VesselType;
use strum::EnumProperty;

impl VesselType {
    pub fn as_chinese(&self) -> &'static str {
        self.into()
    }

    pub fn as_english(&self) -> &'static str {
        self.get_str("English").unwrap()
    }

    pub fn as_full_name(&self) -> &'static str {
        self.get_str("FullName").unwrap()
    }

    pub fn from_chinese(s: &str) -> Option<Self> {
        use strum::IntoEnumIterator;
        Self::iter().find(|vt| vt.as_chinese() == s)
    }

    pub fn from_english(s: &str) -> Option<Self> {
        use strum::IntoEnumIterator;
        Self::iter().find(|vt| vt.as_english() == s)
    }
}

#[cfg(feature = "pyo3")]
#[pyo3::pymethods]
impl VesselType {
    #[pyo3(name = "as_chinese")]
    fn py_as_chinese(&self) -> &'static str {
        self.as_chinese()
    }

    #[pyo3(name = "as_english")]
    fn py_as_english(&self) -> &'static str {
        self.as_english()
    }

    #[pyo3(name = "as_full_name")]
    fn py_as_full_name(&self) -> &'static str {
        self.as_full_name()
    }

    #[staticmethod]
    #[pyo3(name = "from_chinese")]
    fn py_from_chinese(s: String) -> Option<Self> {
        Self::from_chinese(&s)
    }

    #[staticmethod]
    #[pyo3(name = "from_english")]
    fn py_from_english(s: String) -> Option<Self> {
        Self::from_english(&s)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_iter() {
        for i in VesselType::iter() {
            assert!(i.as_english().is_ascii());
            assert_eq!(VesselType::from_chinese(i.as_chinese()), Some(i));
            assert_eq!(VesselType::from_english(i.as_english()), Some(i));
        }
    }

    #[test]
    fn test_names_unique() {
        let all: Vec<_> = VesselType::iter().collect();
        let uniq =
            |f: fn(&VesselType) -> &'static str| all.iter().map(&f).collect::<HashSet<_>>().len();
        assert_eq!(uniq(VesselType::as_chinese), all.len());
        assert_eq!(uniq(VesselType::as_english), all.len());
        assert_eq!(uniq(VesselType::as_full_name), all.len());
    }
}
