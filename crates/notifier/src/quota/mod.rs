//! 额度查询 Provider 集合与调度器。
//!
//! 所有 Provider 的出站请求统一走 [`crate::net::validate_public_url`] 安全校验
//! （仅 http/https；拒绝 localhost/环回/私有/保留地址——这些是公网数据源）。

pub mod custom;
pub mod deepseek;
pub mod glm;
pub mod siliconflow;

use std::time::Duration;

use crate::config::Config;
use crate::ntfy::Notification;

/// 单个 Provider 的一次查询结果。
#[derive(Debug, Clone)]
pub struct QuotaResult {
    pub provider: String,
    /// 展示行（如 "GLM 5小时窗: 42%（重置于 18:30）"）
    pub lines: Vec<String>,
    /// 告警等级（用于判断是否推送告警）
    pub warn_percent: Option<u64>,
}

impl QuotaResult {
    pub fn display_summary(&self) -> String {
        self.lines.join("\n")
    }
}

/// 查询所有启用的 Provider（query 命令与调度器共用）。
pub async fn query_all(config: &Config) -> Vec<Result<QuotaResult, String>> {
    let mut results = Vec::new();
    if config.glm.enabled {
        results.push(glm::fetch(&config.glm).await);
    }
    if config.deepseek.enabled {
        results.push(deepseek::fetch(&config.deepseek).await);
    }
    if config.siliconflow.enabled {
        results.push(siliconflow::fetch(&config.siliconflow).await);
    }
    for provider in &config.custom {
        if provider.enabled {
            results.push(custom::fetch(provider).await);
        }
    }
    results
}

/// 把查询结果渲染成通知（成功行 + 失败行都包含，便于远程诊断）。
pub fn render_notification(results: &[Result<QuotaResult, String>]) -> Notification {
    let mut body_lines = Vec::new();
    for result in results {
        match result {
            Ok(item) => body_lines.push(item.display_summary()),
            Err(error) => body_lines.push(format!("⚠ {error}")),
        }
    }
    if body_lines.is_empty() {
        body_lines.push("没有启用的额度 Provider".to_string());
    }
    Notification {
        title: "额度查询".to_string(),
        body: body_lines.join("\n"),
        tags: vec!["chart".to_string()],
        priority: 3,
    }
}

/// 统一 HTTP 客户端（带超时）
pub(crate) fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

/// 调度器：按各 Provider 配置的间隔轮询并推送。
///
/// 推送策略：
/// - **告警去抖**：某 Provider 用量从低于阈值跨到 ≥ 阈值时推送一次告警；
///   回落或窗口重置（percentage 下降）后重新武装
///
/// 查询结果通过通道发给主程序的推送消费者（toast + ntfy 统一出口）。
pub async fn run_scheduler(config: Config, sender: tokio::sync::mpsc::Sender<Notification>) {
    let interval = scheduler_interval(&config);
    // 各 Provider 告警武装状态（provider 名 → 是否可告警）
    let mut warn_armed: Vec<(String, bool)> = Vec::new();
    loop {
        let results = query_all(&config).await;
        if !results.is_empty() {
            let mut should_push_warn = false;
            let mut summary_lines = Vec::new();
            for result in &results {
                match result {
                    Ok(item) => {
                        summary_lines.push(item.display_summary());
                        if let Some(percent) = item.warn_percent {
                            let threshold = threshold_for(&config, &item.provider);
                            let armed = warn_armed_state(&mut warn_armed, &item.provider, true);
                            if percent >= threshold {
                                if armed {
                                    set_warn_armed(&mut warn_armed, &item.provider, false);
                                    should_push_warn = true;
                                }
                            } else {
                                set_warn_armed(&mut warn_armed, &item.provider, true);
                            }
                        }
                    }
                    Err(error) => summary_lines.push(format!("⚠ {error}")),
                }
            }
            if should_push_warn {
                let notification = Notification {
                    title: "额度告警".to_string(),
                    body: summary_lines.join("\n"),
                    tags: vec!["warning".to_string()],
                    priority: 5,
                };
                let _ = sender.send(notification).await;
            }
        }
        tokio::time::sleep(interval).await;
    }
}

/// 调度间隔取各启用 Provider 的最小值（分钟 → 秒），下限 1 分钟。
fn scheduler_interval(config: &Config) -> Duration {
    let mut minutes: Vec<u64> = Vec::new();
    if config.glm.enabled && !config.glm.api_key.trim().is_empty() {
        minutes.push(config.glm.interval_minutes);
    }
    if config.deepseek.enabled && !config.deepseek.api_key.trim().is_empty() {
        minutes.push(config.deepseek.interval_minutes);
    }
    if config.siliconflow.enabled && !config.siliconflow.api_key.trim().is_empty() {
        minutes.push(config.siliconflow.interval_minutes);
    }
    for provider in &config.custom {
        if provider.enabled && !provider.url.trim().is_empty() {
            minutes.push(provider.interval_minutes);
        }
    }
    let minimum = minutes.iter().copied().min().unwrap_or(5).max(1);
    Duration::from_secs(minimum * 60)
}

fn threshold_for(config: &Config, provider: &str) -> u64 {
    match provider {
        "glm" => config.glm.warn_threshold_percent,
        _ => 80,
    }
}

fn warn_armed_state(states: &mut Vec<(String, bool)>, provider: &str, default: bool) -> bool {
    states
        .iter()
        .find(|(name, _)| name == provider)
        .map(|(_, armed)| *armed)
        .unwrap_or(default)
}

fn set_warn_armed(states: &mut Vec<(String, bool)>, provider: &str, armed: bool) {
    match states.iter_mut().find(|(name, _)| name == provider) {
        Some(entry) => entry.1 = armed,
        None => states.push((provider.to_string(), armed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_result_summary_joins_lines() {
        let result = QuotaResult {
            provider: "glm".into(),
            lines: vec!["5小时窗: 40%".into(), "周窗: 12%".into()],
            warn_percent: Some(40),
        };
        assert_eq!(result.display_summary(), "5小时窗: 40%\n周窗: 12%");
    }

    #[test]
    fn notification_renders_success_and_failure() {
        let results = vec![
            Ok(QuotaResult {
                provider: "glm".into(),
                lines: vec!["5小时窗: 40%".into()],
                warn_percent: Some(40),
            }),
            Err("DeepSeek 查询失败".to_string()),
        ];
        let notification = render_notification(&results);
        assert!(notification.body.contains("5小时窗"));
        assert!(notification.body.contains("DeepSeek 查询失败"));
    }

    #[test]
    fn scheduler_interval_takes_min_enabled() {
        let mut config = Config::default();
        config.glm.api_key = "k".into();
        config.glm.interval_minutes = 5;
        config.deepseek.enabled = true;
        config.deepseek.api_key = "k".into();
        config.deepseek.interval_minutes = 30;
        assert_eq!(scheduler_interval(&config), Duration::from_secs(300));
        // 全部没配 Key → 默认 5 分钟
        config.glm.api_key = String::new();
        config.deepseek.api_key = String::new();
        assert_eq!(scheduler_interval(&config), Duration::from_secs(300));
    }

    #[test]
    fn warn_armed_state_defaults_and_updates() {
        let mut states: Vec<(String, bool)> = Vec::new();
        assert!(warn_armed_state(&mut states, "glm", true));
        set_warn_armed(&mut states, "glm", false);
        assert!(!warn_armed_state(&mut states, "glm", true));
        set_warn_armed(&mut states, "glm", true);
        assert!(warn_armed_state(&mut states, "glm", true));
    }
}
