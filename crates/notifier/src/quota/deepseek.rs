//! DeepSeek 余额查询。
//!
//! 端点：`GET https://api.deepseek.com/user/balance`
//! 认证：`Authorization: Bearer <API_KEY>`。
//! 响应 `balance_infos[].total_balance`（字符串金额，CNY）。

use serde::Deserialize;

use crate::config::BalanceProviderConfig;
use crate::net;
use crate::quota::QuotaResult;

const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com/user/balance";

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    #[serde(default)]
    is_available: bool,
    #[serde(default, rename = "balance_infos")]
    balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, Deserialize)]
struct BalanceInfo {
    #[serde(default)]
    currency: String,
    #[serde(default)]
    total_balance: String,
}

pub async fn fetch(config: &BalanceProviderConfig) -> Result<QuotaResult, String> {
    let api_key = config.api_key.trim();
    if api_key.is_empty() {
        return Err("未配置 API Key（config.toml → deepseek.api_key）".to_string());
    }
    let url = net::validate_public_url(DEEPSEEK_ENDPOINT)?;
    let http = super::http_client()?;
    let response = http
        .get(url)
        .header("Authorization", format!("Bearer {api_key}"))
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
    let parsed: BalanceResponse =
        serde_json::from_str(&body).map_err(|error| format!("响应解析失败: {error}"))?;
    let mut lines = Vec::new();
    for info in &parsed.balance_infos {
        if info.total_balance.is_empty() {
            continue;
        }
        lines.push(format!("DeepSeek 余额: {} {}", info.total_balance, info.currency));
    }
    if lines.is_empty() {
        lines.push(format!("DeepSeek 账户可用: {}", if parsed.is_available { "是" } else { "否" }));
    }
    Ok(QuotaResult { provider: "deepseek".to_string(), lines, warn_percent: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_empty_api_key() {
        let config = BalanceProviderConfig::default();
        assert!(fetch(&config).await.is_err());
    }
}
