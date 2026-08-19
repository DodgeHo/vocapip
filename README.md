# VocaPip

VocaPip 是一个面向 Windows 10/11 x64 的纯离线英语词汇桌宠。应用使用 Tauri 2、React、TypeScript、Rust、SQLite 和 ts-fsrs；学习数据、排程、经验值与设置都保存在本机。

## 已实现

- 透明无边框桌宠，可拖动、置顶或取消置顶，并可一键隐藏到系统托盘。
- 托盘显示/隐藏、暂停提醒、安静模式、单词本、设置和退出。
- 单词学习气泡、双语例句、四档 FSRS 评分、XP、等级、连续学习和防重复刷经验。
- 首次水平定位、T1-T5 分层、80% 舒适区与 20% 挑战区投放。
- SQLite 正式迁移、重启持久化、单词本、收藏、重学、搜索与筛选。
- 提醒、每日配额、安静时段、开机启动、置顶、语言包、本地导入导出和许可证页面。
- 100 条可审计 zh-CN -> en 验证语言包，包含字段级 provenance 和锁定来源。
- 运行时 CSP 禁止网络连接；不集成遥测、更新检查、在线词典、WebSocket 或 AI 请求。

## 开发运行

环境需要 Node.js、Rust MSVC 工具链、WebView2 和 Windows SDK。

```powershell
npm ci
npm run tauri:dev
```

桌宠右上角的眼睛关闭图标会将窗口隐藏到系统托盘。关闭主窗口也只会隐藏；要完全退出，请使用托盘菜单“退出”。

## 验证

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

## 构建

```powershell
npm run tauri:build
```

构建产物位于 `src-tauri/target/release` 及其 `bundle` 子目录。当前项目不包含代码签名证书，因此安装包为未签名产物。

## 本地数据

Windows 默认数据库路径：

```text
%APPDATA%\io.github.dodgeho.vocapip\vocapip.sqlite3
```

从早期 LanguagePet 构建首次启动 VocaPip 时，应用会把旧数据复制到新目录，保留旧数据库不动；旧版备份文件也可以继续导入。导入前会创建安全备份并验证格式；数据库迁移按版本增量执行，不依赖清库重建。

## 许可证

应用代码与项目自制 SVG 桌宠素材采用 GPL-3.0-or-later。语言包集合与每个字段的来源许可、第三方通知和精确版本锁分别记录在 `LICENSES.md`、`THIRD_PARTY_NOTICES.md`、`source-lock.json` 和语言包目录中。
