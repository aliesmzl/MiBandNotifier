# 安装部署指南（Windows 端）

## 前置要求

- Windows 10/11
- Rust 工具链（本项目固定 `x86_64-pc-windows-gnu`；构建需 MinGW-w64 的 gcc，推荐 `scoop install gcc`）
- 手机与电脑同一局域网

## 1. 构建并安装

```powershell
cd MiBandNotifier
cargo build --release

# 安装 zcode hooks（写入 ~/.zcode/cli/config.json，自动备份 .bak，幂等）
.\target\release\MiBandNotifier.exe install-hooks

# 设置开机自启（可选，HKCU Run 键，无需管理员）
.\target\release\MiBandNotifier.exe install-startup
```

> 注意：`install-hooks` 写入的是**当前 exe 的绝对路径**。若移动了 exe 位置，请重新执行 `install-hooks`。

## 2. 部署 ntfy 服务（一次性，需管理员）

ntfy 是局域网通知中转：手机订阅它，消息即可即时到达（前台服务长连接，无需 Google 服务）。

```powershell
# 下载 ntfy.exe（若 GitHub 下载慢，可手动下载放入 tools\ 目录）
powershell -ExecutionPolicy Bypass -File scripts\fetch-ntfy.ps1

# 以管理员身份运行：注册为 Windows 服务（开机自启、崩溃自动恢复）
powershell -ExecutionPolicy Bypass -File scripts\setup-ntfy-service.ps1

# 放行防火墙（管理员）
netsh advfirewall firewall add rule name="MiBandNotifier ntfy" dir=in action=allow protocol=TCP localport=8090
```

验证：浏览器访问 `http://127.0.0.1:8090/v1/health` 应返回 `{"healthy":true}`。

不想装服务也可以：手动运行 `tools\ntfy.exe serve --listen-http 0.0.0.0:8090`（缺点：不开机自启）。

## 3. 启动

```powershell
.\target\release\MiBandNotifier.exe   # 常驻托盘模式
```

托盘图标右键菜单：**立即查询额度 / 暂停推送 / 退出**。

## 4. 配置额度查询

编辑 `%APPDATA%\MiBandNotifier\config.toml`：

```toml
[glm]
enabled = true
api_key = "你的bigmodel API Key"   # bigmodel.cn 控制台 → API Keys 生成
interval_minutes = 5
warn_threshold_percent = 80

[deepseek]
enabled = false
api_key = "sk-..."                  # api.deepseek.com 平台 API Key

[siliconflow]
enabled = false
api_key = "sk-..."

# 自定义第三方 API：任意 http/https 端点 + JSON 指针提取
[[custom]]
name = "我的中转站"
enabled = true
url = "https://api.example.com/v1/usage"
json_pointer = "/data/balance"
interval_minutes = 30

[custom.headers]
Authorization = "Bearer sk-..."
```

改完后托盘菜单点"立即查询额度"验证，或运行 `MiBandNotifier.exe query`。

> 说明：zcode 客户端的 GLM OAuth 凭据是加密存储的，外部程序无法复用；请从
> bigmodel.cn 控制台（GLM Coding Plan 订阅账号即可）生成 API Key。
> 出站请求经过安全校验：仅 http/https，且公网端点不允许指向内网/环回地址。

## 5. 卸载

```powershell
.\target\release\MiBandNotifier.exe uninstall-hooks      # 移除 zcode hooks
.\target\release\MiBandNotifier.exe uninstall-startup    # 取消开机自启
powershell -ExecutionPolicy Bypass -File scripts\setup-ntfy-service.ps1 -Uninstall   # 删 ntfy 服务
```
