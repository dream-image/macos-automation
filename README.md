# macOS Automation

一个关注睡眠省电的 macOS 菜单栏自动化工具。睡眠时自动关闭蓝牙和 Wi-Fi，唤醒后重新开启，让日常使用少一些手动操作。

开发者在排查 Mac 合盖后的异常耗电时发现，睡眠期间的大量后台唤醒指向 Wi-Fi／蓝牙子系统：近 8 天记录的 **10,176 次 DarkWake（后台唤醒）中，10,060 次带有无线相关标记**，占比约 **99%**。这些频繁的后台活动成为不必要耗电的重点排查方向，也促成了这个工具：睡眠时自动关闭无线连接，唤醒后重新开启，以减少相关唤醒机会和待机耗电。

使用 Rust 和 AppKit 构建。

[下载安装](https://github.com/dream-image/macos-automation/releases/latest)

## 功能

- **蓝牙自动化**：睡眠时关闭蓝牙，唤醒后开启。
- **Wi-Fi 自动化**：睡眠时关闭 Wi-Fi，唤醒后开启。
- **接力开关**：从菜单栏直接启用或关闭系统接力（Handoff）。
- **独立配置**：按需启用各项自动化，设置自动保存。
- **菜单栏常驻**：没有主窗口，不占用 Dock 位置。

## 安装

1. 从 [Releases](https://github.com/dream-image/macos-automation/releases/latest) 下载 DMG。
2. 打开镜像，将 `MacOS Automation.app` 拖入“应用程序”。
3. 启动应用，点击菜单栏的齿轮图标。

当前发布包适用于 **Apple Silicon（arm64）**。应用尚未经过 Apple 公证，首次打开可能受到 macOS 安全机制拦截。

## 使用

在菜单中勾选“睡眠自动关闭蓝牙”或“睡眠自动关闭 Wi-Fi”，即可启用对应行为。首次启用蓝牙自动化时，请允许应用访问蓝牙。

Mac 进入睡眠时，应用关闭已启用自动化的无线设备；唤醒约 1 秒后重新开启。**即使设备在睡眠前已被手动关闭，唤醒后也会开启。** 不需要这项行为时，取消对应勾选即可。

接力是独立的手动开关，不随睡眠自动切换。如果在系统设置中修改了接力状态，重启应用可刷新菜单显示。

应用需要保持运行才能执行自动化。需要开机后自动运行时，可自行将它加入系统登录项。

> 省电效果因设备、系统版本及唤醒原因而异。蓝牙和接力控制涉及系统内部接口，macOS 更新后可能需要适配。

## 开发

需要 macOS、Rust 1.88+ 和 Xcode Command Line Tools。

```bash
git clone https://github.com/dream-image/macos-automation.git
cd macos-automation
cargo run --locked
```

启动后在菜单栏操作应用，终端可查看错误输出。选择菜单中的“退出”或按 `Ctrl+C` 结束进程。

编译检查：

```bash
cargo check --locked
```

### 打包

安装 cargo-bundle 并生成应用：

```bash
cargo install cargo-bundle --locked
cargo bundle --release --format osx
```

输出：`target/release/bundle/osx/MacOS Automation.app`。

使用支持 DMG 的 cargo-bundle 版本，也可以生成安装镜像：

```bash
cargo bundle --release --format dmg
```

输出：`target/release/bundle/dmg/MacOS Automation.dmg`。本地构建默认使用本机目标架构，发布前需另行处理应用签名与公证。

### 项目结构

```text
src/
├── main.rs               程序入口
├── app.rs                菜单栏 UI 与睡眠／唤醒事件
├── automation.rs         选项保存与自动化调度
└── settings/
    ├── mod.rs            模块声明
    ├── bluetooth.rs      蓝牙授权与电源控制
    ├── wifi.rs           Wi-Fi 电源控制
    └── handoff.rs        系统接力设置
```

菜单与事件接入、自动化调度、系统操作分别维护；新增行为可在 `settings` 中独立实现，再接入菜单和调度层。应用打包配置位于 `Cargo.toml`，蓝牙权限说明和菜单栏应用标记位于 `Info.plist.ext`。
