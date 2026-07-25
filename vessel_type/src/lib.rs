use strum::{EnumIter, EnumProperty, FromRepr, IntoStaticStr};
#[cfg_attr(feature = "pyo3", pyo3::pyclass(eq, eq_int, from_py_object))]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, FromRepr, IntoStaticStr, EnumIter, EnumProperty)]
pub enum VesselType {
    #[strum(props(Chinese = "航母"))]
    CV,
    #[strum(props(Chinese = "装母"))]
    AV,
    #[strum(props(Chinese = "轻母"))]
    CVL,

    #[strum(props(Chinese = "战列"))]
    BB,
    #[strum(props(Chinese = "战巡"))]
    BC,
    #[strum(props(Chinese = "航战"))]
    BBV,
    #[strum(props(Chinese = "导战"))]
    BBG,
    #[strum(props(Chinese = "大巡"))]
    BG,

    #[strum(props(Chinese = "重巡"))]
    CA,
    #[strum(props(Chinese = "轻巡"))]
    CL,
    #[strum(props(Chinese = "航巡"))]
    CAV,
    #[strum(props(Chinese = "雷巡"))]
    CLT,
    #[strum(props(Chinese = "导巡"))]
    KP,
    #[strum(props(Chinese = "防巡"))]
    CG,

    #[strum(props(Chinese = "驱逐"))]
    DD,
    #[strum(props(Chinese = "导驱"))]
    ASDG,
    #[strum(props(Chinese = "防驱"))]
    AADG,

    #[strum(props(Chinese = "重炮"))]
    BM,

    #[strum(props(Chinese = "潜艇"))]
    SS,
    #[strum(props(Chinese = "炮潜"))]
    SC,
    #[strum(props(Chinese = "导潜"))]
    SSG,

    #[strum(props(Chinese = "补给"))]
    AP,
    #[strum(props(Chinese = "旗舰"))]
    Elite,
    #[strum(props(Chinese = "要塞"))]
    Fortress,
    #[strum(props(Chinese = "港口"))]
    Port,
    #[strum(props(Chinese = "机场"))]
    Airfield,
    #[strum(props(Chinese = "调谐"))]
    NotDef,

    #[strum(props(Chinese = "无"))]
    NO,
}

impl VesselType {
    pub fn as_chinese(&self) -> &'static str {
        self.get_str("Chinese").unwrap()
    }

    pub fn as_english(&self) -> &'static str {
        self.into()
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
    use strum::IntoEnumIterator;

    use super::*;
    #[test]
    fn test_iter() {
        for i in VesselType::iter() {
            assert_eq!(VesselType::from_chinese(i.as_chinese()), Some(i));
            assert_eq!(VesselType::from_english(i.as_english()), Some(i));
        }
    }
}
