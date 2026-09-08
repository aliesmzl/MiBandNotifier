# 手环快应用（M3，进行中）

> 状态：骨架与真机验证指引。快应用是**增强**（手环常驻额度卡片），
> 通知镜像链路（docs/phone-setup.md）已独立可用，不依赖本页。

## 目标

在小米手环 9 Pro 上装一个快应用（RPK），打开即显示 GLM 5 小时窗/周窗用量
进度条与重置倒计时，超阈值长震动。

## 可行性结论（已调研）

- **手环 9 Pro 支持快应用侧载**：AstroBox 完整支持（abox.run 官方兼容表），
  小米运动健康官方 Debug 通道亦可装（我的 → 关于 → Debug → 第三方应用）
- **fetch 联网**：官方文档标注手环"不支持"，但社区项目（米坛 miban9p_tic，
  手环 9 Pro 五子棋在线排行版）实测可用——网络经蓝牙由小米运动健康桥接
- **`system.interconnect`（手机↔手环消息）不可用**：必须配套同签名安卓 App，
  本项目不做安卓端，因此快应用自己 fetch bigmodel 端点
- **震动**：`@system.vibrator` 的 `vibrate({mode:'long'|'short'})` 官方确认
  支持 9/9 Pro
- **屏幕适配**：9 Pro 为 336×480（designWidth: 336），不能照抄手环 10 的 212

## 开发

```powershell
cd band-app
npm install
npm run build    # 产物 dist/*.rpk
```

- 工具链：`aiot-toolkit`（与参考项目 codex-quota-band 一致）
- manifest：`minPlatformVersion: 1200`，features 含 fetch/network/vibrator/storage
- 页面：`pages/index`（额度卡片）、`pages/settings`（API Key 输入，storage 持久化）

## 真机安装

1. AstroBox（https://abox.run/downloads/）连接手环 → 快应用管理 → 导入 rpk
2. 或小米运动健康 → 我的 → 关于 → Debug → 第三方应用 → 选择本地 rpk
3. 装完**完全退出 AstroBox**（它会抢蓝牙连接）
4. 打开手环上的「AI 额度」快应用

## 验证清单（装上后逐项确认）

- [ ] 页面打开不报错、布局不错位（designWidth 336 生效）
- [ ] fetch 能返回数据（若失败 → 记录错误码，降级方案：额度数字走通知推送）
- [ ] API Key 在 settings 页保存后重启快应用仍在
- [ ] 用量 ≥ 阈值时打开快应用有长震动
- [ ] 完全退出 AstroBox 后快应用仍能 fetch（确认桥接来自小米运动健康）
