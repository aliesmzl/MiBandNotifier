//! Windows toast 系统通知（tauri-winrt-notification）。
//!
//! toast 内容只包含事件标题与摘要，不含任何密钥或完整 hook 载荷。

use tauri_winrt_notification::{Duration, Sound, Toast};

/// Toast 的 AppUserModelID。未打包 exe 借用 PowerShell 的 AUMID
/// （系统已注册的合法 ID，tauri-winrt-notification 文档推荐做法）。
const TOAST_APP_ID: &str =
    "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

pub struct ToastContent {
    pub title: String,
    pub body: String,
}

/// 弹出 toast。失败时静默（托盘场景不应因通知失败中断主流程），
/// 仅返回错误供 CLI 模式（notify test）展示。
pub fn show(content: &ToastContent) -> Result<(), String> {
    Toast::new(TOAST_APP_ID)
        .title(&content.title)
        .text1(&content.body)
        .duration(Duration::Short)
        .sound(Some(Sound::Default))
        .show()
        .map_err(|error| format!("toast 显示失败: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_built_with_title_and_body() {
        let content = ToastContent { title: "任务完成".into(), body: "已推送".into() };
        assert_eq!(content.title, "任务完成");
    }
}
