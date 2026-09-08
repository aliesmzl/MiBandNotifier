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

/// 统一 HTTP 客户端（带超时）
fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

/// 调度器：按各 Provider 配置的间隔轮询，超阈值推送告警。
///
/// 告警去抖：每个 Provider 每个告警窗口只在"从低于阈值跨越到不低于阈值"时
/// 推送一次；阈值以下或窗口重置后恢复可再告警。
pub async fn run_scheduler(config: Config) {
    let glm_config = config.glm.clone();
    let glm_interval = Duration::from_secs(glm_config.interval_minutes.max(1) * 60);
    let glm_handle = tokio::spawn(async move {
        let mut warn_armed = true;
        loop {
            if glm_config.enabled && !glm_config.api_key.trim().is_empty() {
                match glm::fetch(&glm_config).await {
                    Ok(result) => {
                        if let Some(percent) = result.warn_percent {
                            let threshold = glm_config.warn_threshold_percent;
                            if percent >= threshold && warn_armed {
                                warn_armed = false;
                                // M2 完整版在此推送告警通知；此处仅打印
                                eprintln!(
                                    "[额度告警] GLM 用量 {}% ≥ 阈值 {}%",
                                    percent, threshold
                                );
                            } else if percent < threshold {
                                warn_armed = true;
                            }
                        }
                    }
                    Err(error) => eprintln!("[额度查询] GLM 失败: {error}"),
                }
            }
            tokio::time::sleep(glm_interval).await;
        }
    });
    let _ = glm_handle.await;
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
}
