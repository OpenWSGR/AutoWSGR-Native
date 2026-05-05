use strum::{EnumIter, EnumProperty, FromRepr, IntoStaticStr};
#[cfg_attr(feature = "pyo3", pyo3::pyclass(eq, eq_int, from_py_object))]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, FromRepr, IntoStaticStr, EnumIter, EnumProperty)]
pub enum VesselType {
    #[strum(props(Chinese = "战列"))]
    BB,
    #[strum(props(Chinese = "航战"))]
    BBV,
    #[strum(props(Chinese = "战巡"))]
    BC,
    #[strum(props(Chinese = "导战"))]
    BBG,
    #[strum(props(Chinese = "大巡"))]
    CBG,

    #[strum(props(Chinese = "航母"))]
    CV,
    #[strum(props(Chinese = "轻母"))]
    CVL,
    #[strum(props(Chinese = "装母"))]
    AV,

    #[strum(props(Chinese = "重巡"))]
    CA,
    #[strum(props(Chinese = "轻巡"))]
    CL,
    #[strum(props(Chinese = "雷巡"))]
    CLT,
    #[strum(props(Chinese = "航巡"))]
    CAV,
    #[strum(props(Chinese = "导巡"))]
    KP,
    #[strum(props(Chinese = "防巡"))]
    CG,

    #[strum(props(Chinese = "重炮"))]
    BM,

    #[strum(props(Chinese = "驱逐"))]
    DD,
    #[strum(props(Chinese = "导驱"))]
    ASDG,
    #[strum(props(Chinese = "防驱"))]
    AADG,

    #[strum(props(Chinese = "潜艇"))]
    SS,
    #[strum(props(Chinese = "炮潜"))]
    SC,

    #[strum(props(Chinese = "补给"))]
    AP,
    #[strum(props(Chinese = "旗舰"))]
    Elite,
    #[strum(props(Chinese = "要塞"))]
    Fortess,
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
    fn test_vessel_type() {
        assert_eq!(VesselType::BB as u8, 0);
        assert_eq!(VesselType::BBV as u8, 1);
        assert_eq!(VesselType::BC as u8, 2);
        assert_eq!(VesselType::BBG as u8, 3);
        assert_eq!(VesselType::CBG as u8, 4);
        assert_eq!(VesselType::CV as u8, 5);
        assert_eq!(VesselType::CVL as u8, 6);
        assert_eq!(VesselType::AV as u8, 7);
        assert_eq!(VesselType::CA as u8, 8);
        assert_eq!(VesselType::CL as u8, 9);
        assert_eq!(VesselType::CLT as u8, 10);
        assert_eq!(VesselType::CAV as u8, 11);
        assert_eq!(VesselType::KP as u8, 12);
        assert_eq!(VesselType::CG as u8, 13);
        assert_eq!(VesselType::BM as u8, 14);
        assert_eq!(VesselType::DD as u8, 15);
        assert_eq!(VesselType::ASDG as u8, 16);
        assert_eq!(VesselType::AADG as u8, 17);
        assert_eq!(VesselType::SS as u8, 18);
        assert_eq!(VesselType::SC as u8, 19);
        assert_eq!(VesselType::AP as u8, 20);
        assert_eq!(VesselType::Elite as u8, 21);
        assert_eq!(VesselType::Fortess as u8, 22);
        assert_eq!(VesselType::Port as u8, 23);
        assert_eq!(VesselType::Airfield as u8, 24);
        assert_eq!(VesselType::NotDef as u8, 25);
        assert_eq!(VesselType::NO as u8, 26);
    }

    #[test]
    fn test_iter() {
        for i in VesselType::iter() {
            println!("{i:?}");
        }
    }
}
