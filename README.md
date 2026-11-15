# AutoWSGR 原生（Rust）后端

实现主包（纯 Python）中不便完成、或对性能有较高要求的功能。目前已提供**游戏截图的图像识别**；
后续计划增加通过 scrcpy/adb 进行的设备控制等。

基于 [huan-yp](https://github.com/huan-yp) 的代码重写。

## 功能

对外以 Python 子包形式提供，均由同一原生扩展 `autowsgr_native._native` 承载：

- `autowsgr_native.recognition`
  - `locate`：在战果截图中定位舰名文字所在的行带。
  - `recognize_enemy`：根据预先切分的字符图块识别敌方舰种。
  - `recognize_map`：识别地图节点的字母（`A`–`J`）。
- `autowsgr_native.vessel_type`
  - `VesselType`：舰种枚举（变体名为中文简称，如 `VesselType.驱逐`），支持中文全称/简称与英文代号互转。

## 结构

- `src/`：核心 Rust 库（`autowsgr_native`），识别功能以子模块组织（`image` / `locator` / `recognize_enemy` / `recognize_map`）。不依赖 pyo3，可独立 `cargo test`。
- `vessel_type/`：独立的舰种枚举 crate（`vessel_type`），开启 `pyo3` feature 时作为 Python 类暴露。
- `python_bindings/`：共享的 pyo3 绑定，将核心库编译为单一原生扩展 `autowsgr_native._native`。
- `python/autowsgr_native/`：Python 包，根包不导出任何内容，各功能以子包形式对外提供。
- `build.rs` + `TemplateData/`：编译期把 `TemplateData/` 下的舰种字模 PNG 打包为 `templates.bin`，供 `recognize_enemy` 运行时加载。
- `vessel_type/build.rs`：编译期从 wiki 同步舰种枚举（见「代码生成」一节）。

## 代码生成（VesselType）

`vessel_type` 的枚举定义（`vessel_type/src/generated.rs`）与 Python 类型存根
（`python/autowsgr_native/vessel_type/__init__.pyi`）由
`vessel_type/build.rs` 在编译期从 [zjsnrwiki「舰船类型」](https://www.zjsnrwiki.com/wiki/舰船类型)
同步，生成物提交进仓库：

- **编译时自动检查**：build.rs 先轻量查询页面 revision 时间戳（api.php 动态响应不带
  HTTP 缓存头，故用 MediaWiki 时间戳作等价条件）。页面时间戳和生成器源码签名都与
  Rust/Python 两份生成物一致才跳过；页面或规则变化都会重新生成，并打印
  `cargo:warning` 提示 review 后提交。源码签名涵盖 `build.rs` 和 `build_support.rs`，
  因此 wiki 未修改时，归一化规则变更也能生效。
- **离线友好**：无法联网时打印 `cargo:warning` 并沿用仓库现有生成物，构建不受影响。
- **纯字符串枚举**：变体名 = wiki 二字中文简称（如 `VesselType.航母`），`FullName` /
  `English` 为 strum 属性，经 `as_full_name()` / `as_english()` 获取；`templates.bin`
  亦按中文名序列化。wiki 新增舰种会自动进枚举，无需登记；不要依赖变体的整数判别值
  （pyo3 暴露的 `int()` 仅供调试，不保证稳定）。
- **缩写归一化**：生成前逐字符将明确支持的西里尔/希腊同形字母及全角 ASCII
  转成 ASCII，保留大小写和 `?`（例如 `КР` → `KP`，`Еlitе` → `Elite`）。
  这是受限字符映射，不是音译；未知字符、非法代号或归一化重名会拒绝更新，
  打印警告并保留原生成物。不会针对某个舰种单独改写缩写。

环境变量：

- `VESSEL_TYPE_OFFLINE`（任意值）：跳过 wiki 检查（CI / 确定性构建）。
- `VESSEL_TYPE_INPUT=<file>`：从本地 JSON（MediaWiki API 响应）生成，不联网（离线重生成/调试）。

已知取舍：cargo 官方建议 build script 只写 `OUT_DIR`，本 build.rs 会重写仓库内生成物
（换取「编译时自动跟随 wiki」）。缓解：仅在内容变化时写盘（mtime 收敛，不会无限重编），
生成物提交进仓库，离线机器的构建结果与提交内容一致。

## 构建

本项目使用 [maturin](https://www.maturin.rs/) 构建 Python 扩展：

```sh
# 安装到当前环境（开发模式）
maturin develop --release

# 或构建 wheel
maturin build --release
```

## 测试

核心库（不依赖 pyo3）可用 `cargo test` 直接运行：

```sh
cargo test --no-default-features
```

（`--no-default-features` 跳过 pyo3 扩展特性，对应 `.cargo/config.toml` 中的 `test-quick` 别名。）
