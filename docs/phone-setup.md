# 手机与手环配置（小米手环 9 Pro）

本项目的手环提醒走**系统通知镜像**链路：任何进入手机通知栏的消息都会被
「小米运动健康」镜像到手环。因此只需让 MiBandNotifier 的通知到达手机。

```
MiBandNotifier (PC) → ntfy 服务 (PC:8090) → 手机 ntfy App 通知
    → 小米运动健康「APP通知提醒」→ 手环震动 + 抬腕显示
```

## 1. 安装 ntfy App

下载地址（任选其一）：

- GitHub Releases APK（推荐，自托管零 Firebase）：https://github.com/binwiederhier/ntfy-android/releases
- F-Droid 版（同样零 Firebase）：https://f-droid.org/packages/io.heckel.ntfy/

> 不要用 Google Play 版连自托管服务器（Play 版对自托管走轮询，送达慢）。

## 2. 订阅通知主题

1. PC 上运行 `MiBandNotifier.exe ntfy-info`，得到订阅地址，例如
   `http://192.168.31.75:8090/mbnacdab6ee47ab`
2. 手机 ntfy App → `+` → 输入地址（`IP:端口/主题`）→ 订阅
3. 测试：PC 上运行 `MiBandNotifier.exe notify test`，手机应立即弹出通知

## 3. 保活设置（关键！）

HyperOS/MIUI 的省电策略会杀后台，两个应用都要设置：

**ntfy App：**
- 设置 → 省电策略 → 无限制
- 最近任务卡片下拉锁定
- 自启动开启（若 ROM 提供）

**小米运动健康：**
- 设置 → 省电策略 → 无限制
- 自启动开启
- 保持在最近任务中

## 4. 手环接收通知

小米运动健康 → 设备（手环 9 Pro）→ **APP通知提醒**：

1. 开启总开关
2. 「管理应用」中勾选 **ntfy**
3. 按需调整震动模式 / 通知优先级

设置后再次运行 `notify test`，手环应震动并在抬腕时显示通知内容。

## 5. 日常使用

保持以下三个常驻即可，手环离开蓝牙范围后回到范围会自动重连：

- PC：MiBandNotifier 托盘（开机自启）+ ntfy Windows 服务
- 手机：ntfy App（前台服务，通知栏有常驻订阅提示）+ 小米运动健康

## 常见问题

| 现象 | 处理 |
|---|---|
| 手机收不到 | 同一局域网？防火墙放行 8090？浏览器访问 `http://PC_IP:8090/v1/health` 验证 |
| ntfy 收到但手环不震 | 小米运动健康是否常驻（保活）；APP通知提醒是否勾选 ntfy；手环勿扰模式 |
| 锁屏后收不到 | ntfy App 电池无限制 + 自启动；锁屏不杀后台 |
| 通知延迟数分钟 | 多半是 ntfy App 被省电策略冻结，检查第 3 步 |
| 换了主题/重装 | `ntfy-info` 重新查看主题，手机重新订阅 |
