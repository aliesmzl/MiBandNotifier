# MiBandNotifier

把 **zcode 任务事件**（完成提醒、等待授权）与 **AI 额度信息**（GLM Coding Plan、DeepSeek、SiliconFlow、自定义 API）推送到**小米手环 9 Pro**。

## 工作原理

```
zcode hooks（Stop / PermissionRequest）
  → MiBandNotifier.exe --hook（短进程，事件落盘，不阻塞 zcode）
  → MiBandNotifier.exe（常驻托盘）
       ├─ Windows toast 系统通知
       └─ ntfy 推送 → 本机 ntfy Windows 服务（0.0.0.0:8090）
            → 手机 ntfy App（局域网订阅，前台服务即时送达）
            → 小米运动健康「APP通知提醒」镜像 → 手环震动 + 显示
```

- 手环端零开发：任何能进手机通知栏的消息都会被小米运动健康镜像到手环
- 事件先落盘后消费：常驻进程不在也不丢事件，下次启动补推
- 出站请求统一安全校验：仅 http/https，公网数据源拒绝内网/环回地址

## 快速开始

### 1. 构建并安装

```powershell
# 需要 Rust（本项目用 windows-gnu 工具链）与 Node.js
cargo build --release
.\target\release\MiBandNotifier.exe install-hooks   # 安装 zcode hooks
```

### 2. 部署 ntfy 服务（一次性）

```powershell
powershell -ExecutionPolicy Bypass -File scripts\fetch-ntfy.ps1
# 以管理员身份运行：
powershell -ExecutionPolicy Bypass -File scripts\setup-ntfy-service.ps1
# 放行防火墙（管理员）：
netsh advfirewall firewall add rule name="MiBandNotifier ntfy" dir=in action=allow protocol=TCP localport=8090
```

### 3. 手机端配置

1. 安装 ntfy App（[GitHub Releases APK](https://github.com/binwiederhier/ntfy-android/releases)，F-Droid 版亦可——自托管服务器不依赖 Google 服务，前台服务即时送达）
2. 运行 `MiBandNotifier.exe ntfy-info` 获取订阅地址，在 ntfy App 中添加订阅
3. 小米运动健康 → 手环 → APP通知提醒 → 勾选 ntfy 应用
4. 给 ntfy App 和小米运动健康开启**自启动**与**电池不受限制**（国产 ROM 保活必需）

### 4. 配置额度查询

编辑 `%APPDATA%\MiBandNotifier\config.toml`：

- `glm.api_key`：bigmodel.cn 控制台 → API Keys 生成（GLM Coding Plan 订阅账号即可）
- `deepseek.api_key` / `siliconflow.api_key`：对应平台 API Key（默认关闭）
- `custom`：任意第三方 API（URL + 请求头 + JSON 指针提取值）

### 5. 验收

```powershell
MiBandNotifier.exe notify test   # PC 弹 toast + 手机/手环收通知
```

之后保持 `MiBandNotifier.exe` 常驻（托盘），zcode 任务完成或需要授权时手环会震动提醒。

## CLI 命令

| 命令 | 作用 |
|---|---|
| （无参数） | 常驻托盘模式 |
| `install-hooks` / `uninstall-hooks` | 安装/卸载 zcode hooks（写入 `~/.zcode/cli/config.json`，自动备份 `.bak`，只增删本项目条目） |
| `query` | 立即查询额度并推送 |
| `notify test` | 发送测试通知 |
| `ntfy-info` | 显示手机订阅地址与防火墙提示 |

## 项目结构

```
crates/notifier   Rust 主程序（托盘、hooks、额度、推送）
band-app/         手环快应用（M3：手环常驻额度卡片，开发中）
scripts/          ntfy 下载与 Windows 服务安装脚本
docs/             详细文档
```

## 状态

- [x] M0 仓库初始化
- [x] M1 核心链路：hook 接收 + toast + ntfy 推送
- [x] M2 额度查询：GLM / DeepSeek / SiliconFlow / 自定义 Provider
- [ ] M3 手环快应用（fetch 真机验证中）
- [ ] M4 开机自启与打包发布

## 参考

- [codex-quota-band](https://github.com/Vincent-hechuan/codex-quota-band)（三端架构参考，本项目借鉴其 hook spool 与托盘模式）
- [ntfy](https://docs.ntfy.sh/)（自托管通知服务）
- [小米 Vela 快应用文档](https://iot.mi.com/vela/quickapp/zh/)

## License

MIT
