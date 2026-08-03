""":mod:`autowsgr_native.recognition` 的类型存根。"""

import numpy as np
from numpy.typing import NDArray

__all__ = ["locate", "recognize_enemy", "recognize_map"]


def locate(img: NDArray[np.uint8]) -> list[list[int]]:
    """定位截图中敌方舰名文字所在的行带。

    Parameters
    ----------
    img:
        形状为 ``(H, W, 3)`` 的 BGR ``uint8`` 数组（例如 ``cv2.imread`` 的输出）。

    Returns
    -------
    list[list[int]]
        由 ``[top, bottom]`` 行索引对组成的列表，每对跨越一行检测到的舰名文字。
    """


def recognize_enemy(images: list[NDArray[np.uint8]]) -> str:
    """根据预先切分好的字符图块识别敌方舰种。

    Parameters
    ----------
    images:
        灰度 ``uint8`` 数组列表，每个形状为 ``(16, 32)``（高 16、宽 32）。

    Returns
    -------
    str
        识别出的英文舰种代号，以单个空格分隔的字符串返回（例如 ``"DD CL BB"``）；
        空白或无法识别的槽位记为 ``NO``。
    """


def recognize_map(img: NDArray[np.uint8]) -> str:
    """根据图标识别地图节点。

    Parameters
    ----------
    img:
        形状为 ``(H, W, 3)`` 的 BGR ``uint8`` 数组。

    Returns
    -------
    str
        分类节点的单字符字符串（``A``-``J``）；识别失败时返回 NUL 字符（``'\\x00'``）。
    """
