# macOS Automation

一个使用 Rust 和 AppKit 开发的 macOS 菜单栏自动化工具。

## 功能

- Mac 进入睡眠时自动关闭蓝牙
- Mac 进入睡眠时自动关闭 Wi-Fi
- Mac 唤醒后自动恢复已启用的蓝牙和 Wi-Fi
- 菜单栏直接启用或停用各项行为
- 自动保存设置

## 开发环境

- macOS
- Rust 1.85 或更高版本

## 运行

```bash
cargo run
```

## 打包

安装 `cargo-bundle`：

```bash
cargo install cargo-bundle
```

生成 release 应用和 DMG：

```bash
cargo bundle --release
```

输出位置：

```text
target/release/bundle/osx/MacOS Automation.app
target/release/bundle/dmg/MacOS Automation.dmg
```

请通过 `.app` 启动应用。直接运行 `target/release/macos-automation` 会由终端承载进程。

## 权限

启用蓝牙行为时，应用会申请系统蓝牙权限。Wi-Fi 电源控制在系统需要时可能要求管理员授权。

## 代码结构

```text
src/app.rs                  菜单栏 UI 和睡眠、唤醒事件
src/automation.rs           设置存储与行为调度
src/settings/bluetooth.rs   蓝牙控制
src/settings/wifi.rs        Wi-Fi 控制
```
