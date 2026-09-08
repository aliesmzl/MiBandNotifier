//! SiliconFlow 余额查询。
//!
//! 端点：`GET https://api.siliconflow.cn/v1/user/info`
//! 认证：`Authorization: Bearer <API_KEY>`。
//! 响应 `data.balance`（总余额）与 `data.chargeBalance`（充值余额），字符串金额。

use serde::Deserialize;

use crate::config::BalanceProviderConfig;
use crate::net;
use crate::quota::QuotaResult;

const SILICONFLOW_ENDPOINT: &str = "https://api.siliconflow.cn/v1/user/info";

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    data: Option<UserData>,
}

#[derive(Debug, Deserialize)]
struct UserData {
    #[serde(default)]
    name: String,
    #[serde(default)]
    balance: String,
    #[serde(default)]
    charge_balance: Option<String>,
}

pub async fn fetch(config: &BalanceProviderConfig) -> Result<QuotaResult, String> {
    let api_key = config.api_key.trim();
    if api_key.is_empty() {
        return Err("未配置 API Key（config.toml → siliconflow.api_key）".to_string());
    }
    let url = net::validate_public_url(SILICONFLOW_ENDPOINT)?;
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
    let parsed: UserInfoResponse =
        serde_json::from_str(&body).map_err(|error| format!("响应解析失败: {error}"))?;
    let data = parsed
        .data
        .ok_or_else(|| format!("响应缺少 data: {}", parsed.message))?;
    let mut lines = vec![format!("SiliconFlow 余额: {}", data.balance)];
    if let Some(charge) = &data.charge_balance {
        if !charge.is_empty() {
            lines.push(format!("其中充值余额: {charge}"));
        }
    }
    Ok(QuotaResult { provider: "siliconflow".to_string(), lines, warn_percent: None })
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
