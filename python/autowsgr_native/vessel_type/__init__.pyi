""":mod:`autowsgr_native.vessel_type` 的类型存根。"""

from typing import ClassVar

__all__ = ["VesselType"]


class VesselType:
    """从游戏截图中识别出的舰船或设施类型。

    各变体以类属性形式暴露（如 ``VesselType.DD``）。使用
    :meth:`as_english` / :meth:`as_chinese` 获取其字符串代号，使用
    :meth:`from_english` / :meth:`from_chinese` 将代号解析回变体。
    """

    CV: ClassVar[VesselType]
    """航母"""
    AV: ClassVar[VesselType]
    """装母"""
    CVL: ClassVar[VesselType]
    """轻母"""

    BB: ClassVar[VesselType]
    """战列"""
    BC: ClassVar[VesselType]
    """战巡"""
    BBV: ClassVar[VesselType]
    """航战"""
    BBG: ClassVar[VesselType]
    """导战"""
    BG: ClassVar[VesselType]
    """大巡"""

    CA: ClassVar[VesselType]
    """重巡"""
    CL: ClassVar[VesselType]
    """轻巡"""
    CAV: ClassVar[VesselType]
    """航巡"""
    CLT: ClassVar[VesselType]
    """雷巡"""
    KP: ClassVar[VesselType]
    """导巡"""
    CG: ClassVar[VesselType]
    """防巡"""

    DD: ClassVar[VesselType]
    """驱逐"""
    ASDG: ClassVar[VesselType]
    """导驱"""
    AADG: ClassVar[VesselType]
    """防驱"""

    BM: ClassVar[VesselType]
    """重炮"""

    SS: ClassVar[VesselType]
    """潜艇"""
    SC: ClassVar[VesselType]
    """炮潜"""
    SSG: ClassVar[VesselType]
    """导潜"""

    AP: ClassVar[VesselType]
    """补给"""
    Elite: ClassVar[VesselType]
    """旗舰"""
    Fortress: ClassVar[VesselType]
    """要塞"""
    Port: ClassVar[VesselType]
    """港口"""
    Airfield: ClassVar[VesselType]
    """机场"""
    NotDef: ClassVar[VesselType]
    """调谐"""

    NO: ClassVar[VesselType]
    """无"""

    def as_chinese(self) -> str:
        """该舰种的中文名（如 ``"驱逐"``）。"""

    def as_english(self) -> str:
        """该舰种的英文代号（如 ``"DD"``）。"""

    @classmethod
    def from_chinese(cls, s: str) -> VesselType | None:
        """按中文名解析为变体，无法识别时返回 ``None``。"""

    @classmethod
    def from_english(cls, s: str) -> VesselType | None:
        """按英文代号解析为变体，无法识别时返回 ``None``。"""
