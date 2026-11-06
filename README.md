# VocaPip

> 纯离线英语词汇桌宠 / An offline English vocabulary desktop pet for Windows

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Platform: Windows x64](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D4.svg)](https://github.com/DodgeHo/vocapip)

## 中文

VocaPip 是一个面向 Windows 10/11 x64 的纯离线英语词汇桌宠。应用使用 Tauri 2、React、TypeScript、Rust、SQLite 和 ts-fsrs；学习数据、排程、经验值与设置都保存在本机。

### 功能

- 透明无边框桌宠，可拖动、置顶或取消置顶，并可一键隐藏到系统托盘。
- 托盘显示/隐藏、暂停提醒、安静模式、单词本、设置和退出。
- 单词学习气泡、双语例句、四档 FSRS 评分、XP、等级、连续学习和防重复刷经验。
- 首次水平定位、T1-T5 分层、80% 舒适区与 20% 挑战区投放。
- SQLite 正式迁移、重启持久化、单词本、收藏、重学、搜索与筛选。
- 提醒、每日配额、安静时段、开机启动、置顶、语言包、本地导入导出和许可证页面。
- 100 条可审计 zh-CN -> en 验证语言包，包含字段级 provenance 和锁定来源。
- 运行时 CSP 禁止网络连接；不集成遥测、更新检查、在线词典、WebSocket 或 AI 请求。

### 下载

Windows 10/11 x64 安装包、便携版和 SHA-256 校验文件见 [VocaPip 0.3.0 Release](https://github.com/DodgeHo/vocapip/releases/tag/v0.3.0)。当前二进制未签名，Windows SmartScreen 可能显示警告；运行前请核对 Release 中的 `SHA256SUMS.txt`。

### 开发运行

环境需要 Node.js、Rust MSVC 工具链、WebView2 和 Windows SDK。

```powershell
npm ci
npm run tauri:dev
```

桌宠右上角的眼睛关闭图标会将窗口隐藏到系统托盘。关闭主窗口也只会隐藏；要完全退出，请使用托盘菜单“退出”。

### 验证

```powershell
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run test:e2e
cargo fmt --manifest-path src-tauri\Cargo.toml -- --check
cargo clippy --manifest-path src-tauri\Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri\Cargo.toml --locked
```

内容管线验证命令见 `content-pipeline/README.md`。

### 构建

```powershell
npm run tauri:build
```

构建产物位于 `src-tauri/target/release` 及其 `bundle` 子目录。当前项目不包含代码签名证书，因此安装包为未签名产物。

### 本地数据

Windows 默认数据库路径：

```text
%APPDATA%\io.github.dodgeho.vocapip\vocapip.sqlite3
```

从早期 LanguagePet 构建首次启动 VocaPip 时，应用会把旧数据复制到新目录，保留旧数据库不动；旧版备份文件也可以继续导入。导入前会创建安全备份并验证格式；数据库迁移按版本增量执行，不依赖清库重建。

## English

VocaPip is a fully offline English vocabulary desktop pet for Windows 10/11 x64. It is built with Tauri 2, React, TypeScript, Rust, SQLite, and ts-fsrs. Learning data, scheduling, experience points, and settings stay on the local machine.

### Features

- A transparent, borderless desktop pet that can be dragged, kept always on top, or hidden to the system tray.
- Tray controls for show/hide, pausing reminders, quiet mode, the vocabulary library, settings, and exit.
- Word-learning bubbles, bilingual example sentences, four FSRS ratings, XP, levels, streaks, and anti-farming XP protection.
- Initial placement assessment, T1-T5 tiers, and an 80% comfort / 20% challenge mix.
- SQLite migrations, restart persistence, favorites, relearning, search, and filters.
- Reminders, daily quotas, quiet hours, launch at startup, language packs, local import/export, and a license page.
- A 100-entry auditable zh-CN -> en validation pack with field-level provenance and locked sources.
- Runtime CSP blocks network access. There is no telemetry, update checker, online dictionary, WebSocket, or AI request integration.

### Download

Windows 10/11 x64 installers, the portable build, and SHA-256 checksums are available in the [VocaPip 0.3.0 Release](https://github.com/DodgeHo/vocapip/releases/tag/v0.3.0). The binaries are currently unsigned, so Windows SmartScreen may show a warning. Verify `SHA256SUMS.txt` before running a release build.

### Development

You need Node.js, the Rust MSVC toolchain, WebView2, and the Windows SDK.

```powershell
npm ci
npm run tauri:dev
```

The eye icon in the top-right corner hides the pet to the system tray. Closing the main window also hides it; use the tray menu's Exit command to quit completely.

### Build

```powershell
npm run tauri:build
```

Build output is written to `src-tauri/target/release` and its `bundle` subdirectory. The project does not include a code-signing certificate, so installer artifacts are unsigned.

## License and Third-Party Content

The application code and project-created SVG pet assets are licensed under [GPL-3.0-or-later](LICENSE). The language-pack collection and the license or provenance for each source field are documented in `LICENSES.md`, `THIRD_PARTY_NOTICES.md`, `source-lock.json`, and the language-pack directories. Review those files before redistributing modified or bundled content.
