//! GLM Coding Plan 额度查询。
//!
//! 端点（控制台同款监控接口，社区工具 UsageBoard / CodexBar 印证）：
//! `GET https://open.bigmodel.cn/api/monitor/usage/quota/limit`
//! 认证：`Authorization: Bearer <API_KEY>`（bigmodel 控制台生成的 API Key，
//! 注意不是 zcode 的 OAuth 凭据）。
//!
//! 响应 `data.limits[]` 中：
//! - `limitType`: TOKENS_LIMIT / CREDIT_LIMIT（Coding Plan 主/次窗口）
//! - 窗口时长：`unit==3&&number==5` → 5 小时窗；`unit==6&&number==1` → 周窗
//! - `percentage`（已用百分比）、`nextResetTime`（epoch 秒或毫秒，自适应）

use serde::Deserialize;

use crate::config::GlmConfig;
use crate::net;
use crate::quota::QuotaResult;

const GLM_ENDPOINT: &str = "https://open.bigmodel.cn/api/monitor/usage/quota/limit";

#[derive(Debug, Deserialize)]
struct GlmResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    code: i64,
    #[serde(default)]
    msg: String,
    #[serde(default)]
    data: Option<GlmData>,
}

#[derive(Debug, Deserialize)]
struct GlmData {
    #[serde(default)]
    limits: Vec<GlmLimit>,
    #[serde(default)]
    plan_name: Option<String>,
    #[serde(default)]
    plan: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GlmLimit {
    #[serde(default, rename = "limitType")]
    limit_type: Option<String>,
    #[serde(default)]
    unit: Option<i64>,
    #[serde(default)]
    number: Option<i64>,
    #[serde(default)]
    percentage: Option<f64>,
    #[serde(default)]
    next_reset_time: Option<i64>,
}

/// 查询 GLM Coding Plan 用量。
pub async fn fetch(config: &GlmConfig) -> Result<QuotaResult, String> {
    let api_key = config.api_key.trim();
    if api_key.is_empty() {
        return Err("未配置 API Key（config.toml → glm.api_key）".to_string());
    }
    let url = net::validate_public_url(GLM_ENDPOINT)?;
    let http = super::http_client()?;
    let response = http
        .get(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("请求失败: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取响应失败: {error}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    let parsed: GlmResponse =
        serde_json::from_str(&body).map_err(|error| format!("响应解析失败: {error}"))?;
    if !parsed.success || parsed.code != 0 {
        return Err(format!("接口错误 code={} msg={}", parsed.code, parsed.msg));
    }
    let data = parsed.data.ok_or_else(|| "响应缺少 data".to_string())?;
    Ok(build_result(&data))
}

fn build_result(data: &GlmData) -> QuotaResult {
    let mut lines = Vec::new();
    let mut warn_percent: Option<u64> = None;
    let plan = data.plan_name.as_deref().or(data.plan.as_deref()).unwrap_or("Coding Plan");
    for limit in &data.limits {
        let Some(kind) = limit.limit_type.as_deref() else {
            continue;
        };
        if kind != "TOKENS_LIMIT" && kind != "CREDIT_LIMIT" {
            continue;
        }
        let window = window_label(limit.unit, limit.number);
        let Some(percent) = limit.percentage else {
            continue;
        };
        let reset = limit
            .next_reset_time
            .map(|value| format_reset_time(value))
            .unwrap_or_else(|| "未知".to_string());
        // 显示与告警使用同一个取整值，避免两者不一致
        let percent_rounded = (percent + 0.5).floor() as u64;
        lines.push(format!("{plan} {window}: {percent_rounded}%（重置于 {reset}）"));
        if warn_percent.is_none() || percent_rounded > warn_percent.unwrap_or(0) {
            warn_percent = Some(percent_rounded);
        }
    }
    if lines.is_empty() {
        lines.push(format!("{plan}: 无可用窗口数据"));
    }
    QuotaResult { provider: "glm".to_string(), lines, warn_percent }
}

/// unit+number → 窗口标签
fn window_label(unit: Option<i64>, number: Option<i64>) -> String {
    match (unit, number) {
        (Some(3), Some(5)) => "5小时窗".to_string(),
        (Some(6), Some(1)) => "周窗".to_string(),
        (Some(5), Some(1)) => "月窗".to_string(),
        (Some(unit), Some(number)) if unit > 0 && number > 0 => {
            format!("{number}×unit{unit}窗")
        }
        _ => "窗口".to_string(),
    }
}

/// nextResetTime 秒/毫秒自适应（>10^10 视为毫秒），转本地时间 HH:mm。
fn format_reset_time(raw: i64) -> String {
    let millis = if raw > 10_000_000_000 { raw } else { raw * 1000 };
    let datetime = chrono::DateTime::from_timestamp_millis(millis)
        .unwrap_or_else(chrono::Utc::now);
    datetime
        .with_timezone(&chrono::Local)
        .format("%m-%d %H:%M")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> GlmData {
        GlmData {
            limits: vec![
                GlmLimit {
                    limit_type: Some("TOKENS_LIMIT".into()),
                    unit: Some(3),
                    number: Some(5),
                    percentage: Some(42.5),
                    next_reset_time: Some(1_780_000_000),
                },
                GlmLimit {
                    limit_type: Some("CREDIT_LIMIT".into()),
                    unit: Some(6),
                    number: Some(1),
                    percentage: Some(12.0),
                    next_reset_time: None,
                },
                GlmLimit {
                    limit_type: Some("TIME_LIMIT".into()),
                    unit: Some(2),
                    number: Some(1),
                    percentage: Some(99.0),
                    next_reset_time: None,
                },
            ],
            plan_name: Some("GLM Coding Plan".into()),
            plan: None,
        }
    }

    #[test]
    fn parses_windows_and_ignores_time_limit() {
        let result = build_result(&sample_data());
        assert_eq!(result.lines.len(), 2);
        assert!(result.lines[0].contains("5小时窗"));
        assert!(result.lines[0].contains("43%"));
        assert!(result.lines[1].contains("周窗"));
        assert_eq!(result.warn_percent, Some(43));
    }

    #[test]
    fn reset_time_handles_seconds_and_millis() {
        let seconds = format_reset_time(1_780_000_000);
        let millis = format_reset_time(1_780_000_000_000);
        assert!(!seconds.is_empty());
        assert!(!millis.is_empty());
    }

    #[test]
    fn empty_limits_fall_back_to_plan_line() {
        let data = GlmData { limits: vec![], plan_name: None, plan: None };
        let result = build_result(&data);
        assert_eq!(result.lines.len(), 1);
        assert!(result.lines[0].contains("无可用窗口数据"));
    }

    #[tokio::test]
    async fn rejects_empty_api_key() {
        let config = GlmConfig::default();
        let error = fetch(&config).await.unwrap_err();
        assert!(error.contains("API Key"));
    }
}
