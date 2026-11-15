# Page-Last-Modified: 2026-09-28T16:42:48Z
# Generator-Signature: fnv1a64:263c84c4c1d00082
""":mod:`autowsgr_native.vessel_type` 的类型存根。

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

    航母: ClassVar[VesselType]
    """航空母舰 / CV"""
    装母: ClassVar[VesselType]
    """装甲航母 / AV"""
    轻母: ClassVar[VesselType]
    """轻型航母 / CVL"""
    战列: ClassVar[VesselType]
    """战列舰 / BB"""
    战巡: ClassVar[VesselType]
    """战列巡洋舰 / BC"""
    航战: ClassVar[VesselType]
    """航空战列舰 / BBV"""
    导战: ClassVar[VesselType]
    """反舰导弹战列舰 / BBG"""
    防战: ClassVar[VesselType]
    """防空导弹战列舰 / BG"""
    大巡: ClassVar[VesselType]
    """导弹大型巡洋舰 / CBG"""
    重巡: ClassVar[VesselType]
    """重巡洋舰 / CA"""
    轻巡: ClassVar[VesselType]
    """轻巡洋舰 / CL"""
    航巡: ClassVar[VesselType]
    """航空巡洋舰 / CAV"""
    雷巡: ClassVar[VesselType]
    """重雷装巡洋舰 / CLT"""
    导巡: ClassVar[VesselType]
    """反舰导弹巡洋舰 / KP"""
    防巡: ClassVar[VesselType]
    """防空导弹巡洋舰 / CG"""
    驱逐: ClassVar[VesselType]
    """驱逐舰 / DD"""
    导驱: ClassVar[VesselType]
    """反舰导弹驱逐舰 / ASDG"""
    防驱: ClassVar[VesselType]
    """防空导弹驱逐舰 / AADG"""
    重炮: ClassVar[VesselType]
    """浅水重炮舰 / BM"""
    潜艇: ClassVar[VesselType]
    """潜水艇 / SS"""
    炮潜: ClassVar[VesselType]
    """重炮潜艇 / SC"""
    导潜: ClassVar[VesselType]
    """导弹潜艇 / SSG"""
    补给: ClassVar[VesselType]
    """补给舰 / AP"""

    旗舰: ClassVar[VesselType]
    """旗舰 / Elite"""
    要塞: ClassVar[VesselType]
    """岸防要塞 / Fortress"""
    港口: ClassVar[VesselType]
    """海军基地 / Port"""
    机场: ClassVar[VesselType]
    """空军基地 / Airfield"""
    调谐: ClassVar[VesselType]
    """调谐舰 / ?"""

    无: ClassVar[VesselType]
    """无 / NO"""

    def as_chinese(self) -> str:
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
