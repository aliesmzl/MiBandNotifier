# MiBandNotifier 项目 Agent 规范

本文件是给 AI Agent 的工作入口。开始任何修改前先读本文件；需要部署/手机端细节时读
`docs/setup.md`、`docs/phone-setup.md`，涉及手环快应用时读 `docs/band-app.md`。
历史决策与调研结论不在本文件展开，见 Git 提交信息与 docs。

## 1. 项目定位与架构

把 **zcode 任务事件**（完成/等待授权）与 **AI 额度信息**推送到**小米手环 9 Pro**。
目标机型与链路一经确认不要更改：

```
zcode hooks (Stop / PermissionRequest, process 类型)
  → MiBandNotifier.exe --hook（短进程：stdin JSON → spool 落盘 → 立即退出）
  → 常驻托盘（Win32 消息泵主线程 + tokio 后台，spool 轮询）
      → toast（tauri-winrt-notification）+ ntfy 推送（mpsc 通道统一消费）
        → ntfy Windows 服务（0.0.0.0:8090）→ 手机 ntfy App（前台服务即时送达）
          → 小米运动健康「APP通知提醒」镜像 → 手环 9 Pro 震动 + 显示
```

- 架构决策（用户已确认）：**通知镜像主链路 + 手环快应用增强；不做安卓 App**
  （`system.interconnect` 需要同签名安卓端，已排除）；Windows 端 Rust。
- ntfy 以 Windows 服务常驻（`scripts/setup-ntfy-service.ps1`），Rust 端只做 HTTP 推送。
- 手环快应用是增强不是依赖：通知链路独立完整可用。9 Pro 屏幕为 **336×480，
  designWidth 必须 336**（不能照抄手环 10 的 212）；快应用页面必须是目录结构
  `pages/<name>/<name>.ux`。
- GLM 额度端点 `https://open.bigmodel.cn/api/monitor/usage/quota/limit` 为控制台
  逆向 API，无官方文档：解析必须容错（字段全 Option），接口变更时先报错明确再适配。
  认证用 bigmodel 控制台 API Key；zcode 的 OAuth 凭据是 `enc:v1` 加密的，不可复用。

## 2. 不可违反的安全与隐私边界

- **出站请求统一走 `net::validate_public_url`**：仅 http/https；公网数据源（额度
  Provider，含用户自定义 URL）拒绝 localhost/环回/私有/保留地址。唯一环回豁免是
  ntfy 推送（`net::validate_ntfy_url`），因为目标是用户配置的本机/局域网自有服务。
  新增任何网络请求必须过这两个函数，并补对应单测。
- **API Key 只存 `%APPDATA%\MiBandNotifier\config.toml`**：不进 git、不进日志、
  不进通知内容、不进 toast。
- **hook 载荷最小化**：`--hook` 短进程只把 zcode 传入的 stdin JSON 原样落盘 spool
  （上限 32 KiB）；通知正文只允许事件标题 + `summary_line()` 提取的短摘要
  （≤60 字符）。不得把提示词、文件路径、完整回复写进日志或通知。
- **短进程绝不阻塞 zcode**：`--hook` 任何失败都静默 `exit 0`（超时 3000ms）。
- **不在 Rust 里拉子进程**：`Command::new` 会被 Mimosa 钩子误判拦截，且架构上
  外部工具（ntfy）已服务化。新需求若看似需要子进程，改用 HTTP/API/服务脚本。
- **暂停推送只在推送消费者一处生效**（`start_background` 的消费任务）；
  新增推送路径必须走 mpsc 通道，不要绕过它直接调 `push_notification`。
- 默认只在可信局域网运行；不新增云端中转、遥测、广告或自动崩溃上报。

## 3. 代码边界与关键目录

| 目录 | 责任 | 主要技术 |
| --- | --- | --- |
| `crates/notifier/src/main.rs` | CLI 分发、Win32 托盘/菜单、消息泵、推送消费者 | Rust 2024、windows-sys、tokio |
| `crates/notifier/src/quota/` | 额度 Provider（glm/deepseek/siliconflow/custom）+ 调度器 | reqwest（rustls ring provider） |
| `crates/notifier/src/hook.rs` | hook 事件 spool（落盘/消费/摘要提取） | serde_json |
| `crates/notifier/src/zcode_hooks.rs` | 安装/卸载 zcode hooks（owner-marker 幂等合并） | serde_json |
| `crates/notifier/src/net.rs` | 出站 URL 安全校验（SSRF 防护） | — |
| `crates/notifier/src/ntfy.rs` / `toast.rs` | ntfy 客户端 / toast 封装 | reqwest / tauri-winrt-notification |
| `crates/notifier/src/startup.rs` | 开机自启（HKCU Run 键） | windows-sys Registry |
| `band-app/` | 手环 9 Pro 快应用（额度卡片 + 设置页） | Vela 快应用、aiot-toolkit |
| `scripts/` | ntfy 下载/服务安装/打包 | PowerShell 5.1 |
| `docs/` | 部署、手机端、手环真机验收文档 | Markdown |

Win32 部分遵循固定模式：主线程只跑消息泵（`GetMessageW` 循环），tokio Runtime 放
后台；后台需要改托盘（tooltip）时通过自定义消息（`TRAY_UPDATE`）+ 静态文本回主线程，
不要跨线程直接调 `Shell_NotifyIconW`。edition 2024：`unsafe fn` 体内的 unsafe 调用
要包 `unsafe {}` 块。

## 4. 开始任务时的固定流程

1. `git status --short` 确认工作区；读本文件与 `README.md`。
2. 先定位现有实现和测试，再做最小修改；不要为“看起来更规范”重写无关模块。
3. 新功能先写/补单元测试（参考 `net.rs`、`quota/mod.rs` 的测试风格），实现后
   `cargo test` 必须全绿。
4. 涉及通知内容/新 Provider 时，自查第 2 节边界（校验、脱敏、Key 去向）。
5. 交付说明包含：改动、测试结果、真机仍需验证的部分、Git 状态。

## 5. 常用命令（本机构建硬约束）

```bash
# Rust 构建/测试：必须先把 scoop 的 MinGW 加入 PATH
# （rust-toolchain.toml 固定 x86_64-pc-windows-gnu；本机无 MSVC link.exe，
#   rustup 自带 self-contained 工具缺 as/dlltool）
export PATH="$HOME/scoop/apps/gcc/current/bin:$PATH"
cargo build --release
cargo test
```

```powershell
# 手环快应用（产物 .temp_band-app\dist\*.rpk，会同步到 band-app\dist）
cd band-app; npm install; npm run build

# 打包发布（exe + scripts + docs + tools\ntfy.exe → dist\*.zip）
powershell -ExecutionPolicy Bypass -File scripts\package.ps1 -Root <仓库绝对路径>
```

- Git Bash 里跑 cargo 会命中 `/usr/bin/link.exe`（coreutils），必须如上加 PATH 前缀
  或用 `cmd //c`。
- **PowerShell 脚本含中文必须带 UTF-8 BOM**：Write 工具写出的是无 BOM UTF-8，
  本机 PS 5.1 会按 GBK 解析导致解析错位（变量莫名 null、输出乱码）。写完执行
  `printf '\xef\xbb\xbf' | cat - 文件 > 临时 && mv 临时 文件` 补 BOM，
  验证 `head -c 3 文件 | od -An -tx1` 为 `ef bb bf`。
- `main()` 启动时安装 rustls ring CryptoProvider（reqwest 用 no-provider 特性），
  新增二进制入口或测试里构建 reqwest Client 时同样要装（见 `ntfy.rs` 测试）。

## 6. 版本号一致性

`Cargo.toml`(workspace) `version`、`band-app/src/manifest.json` 的
`versionName`/`versionCode`、`scripts/package.ps1` 的 `$version` 必须同步修改。
版本号只递增，不回退。

## 7. 测试与真机验收

- 自动测试覆盖：SSRF 校验正反例、Provider 解析容错、spool 往返/截断/清理、
  hooks 安装幂等与保留他人条目、看板 tooltip 拼接截断、退避封顶。
- 自动测试不能代替真机验收。真机项清单见 `docs/band-app.md`（快应用 fetch 联网
  是关键风险项）与 `docs/phone-setup.md`（手机镜像链路）。
- 常规里程碑开发可直接 commit/push（用户已授权）；**GitHub Release 发布、
  正式签名**需用户明确确认后执行。
