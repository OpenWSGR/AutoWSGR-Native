"""AutoWSGR 原生（Rust）后端。

承载主包（纯 Python）中不便实现的功能。当前提供图像识别（见
:mod:`autowsgr_native.recognition` 与 :mod:`autowsgr_native.vessel_type`），
后续计划增加通过 scrcpy/adb 的设备控制等。

根包本身刻意不导出任何内容，各功能以子包形式对外提供，便于后续按功能追加后端。
"""
