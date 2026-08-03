# AutoWSGR 原生（Rust）后端

实现主包（纯 Python）中不便完成的功能，如图像识别、通过 scrcpy/adb 进行设备控制等。

基于 [huan-yp](https://github.com/huan-yp) 的代码重写。

## 结构

- `src/`：核心 Rust 库（`autowsgr_native`），各功能以子模块形式组织（如 `recognition`，后续将增加 `scrcpy`、`adb` 等）。不依赖 pyo3，可独立 `cargo test`。
- `python_bindings/`：共享的 pyo3 绑定，将核心库编译为单一原生扩展 `autowsgr_native._native`。
- `python/autowsgr_native/`：Python 包，根包不导出任何内容，各功能以子包形式对外提供（如 `autowsgr_native.recognition`）。
