//! 自定义 Provider：任意 http/https 端点 + 请求头 + JSON 指针提取。
//!
//! 满足"第三方 API"扩展需求。URL 经 [`crate::net::validate_public_url`] 校验
//! （仅 http/https；拒绝 localhost/环回/私有/保留地址），请求头由用户配置提供。

use serde_json::Value;

use crate::config::CustomProviderConfig;
use crate::net;
use crate::quota::QuotaResult;

pub async fn fetch(config: &CustomProviderConfig) -> Result<QuotaResult, String> {
    let name = if config.name.trim().is_empty() { "自定义" } else { config.name.trim() };
    if config.url.trim().is_empty() {
        return Err(format!("{name}: 未配置 URL"));
    }
    let url = net::validate_public_url(config.url.trim())?;
    let http = super::http_client()?;
    let mut request = http.get(url);
    for (key, value) in &config.headers {
        if key.eq_ignore_ascii_case("host") || key.eq_ignore_ascii_case("content-length") {
            continue;
        }
        request = request.header(key, value);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("{name}: 请求失败: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("{name}: 读取响应失败: {error}"))?;
    if !status.is_success() {
        return Err(format!("{name}: HTTP {status}"));
    }
    let value: Value =
        serde_json::from_str(&body).map_err(|error| format!("{name}: 响应不是 JSON: {error}"))?;
    let pointer = config.json_pointer.trim();
    let line = if pointer.is_empty() {
        // 未配置指针：展示顶层键值概览
        summarize_object(&value)
    } else {
        let extracted = value
            .pointer(pointer)
            .ok_or_else(|| format!("{name}: JSON 指针 {pointer} 未命中"))?;
        format!("{name}: {}", plain_value(extracted))
    };
    Ok(QuotaResult {
        provider: config.name.clone(),
        lines: vec![line],
        warn_percent: None,
    })
}

fn summarize_object(value: &Value) -> String {
    match value {
        Value::Object(map) if !map.is_empty() => {
            let preview: Vec<String> = map
                .iter()
                .take(3)
                .map(|(key, item)| format!("{key}={}", plain_value(item)))
                .collect();
            preview.join(", ")
        }
        Value::Object(_) => "{}".to_string(),
        other => plain_value(other),
    }
}

fn plain_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(items) => format!("[{}项]", items.len()),
        Value::Object(map) => format!("{{{}键}}", map.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn provider(url: &str, pointer: &str) -> CustomProviderConfig {
        CustomProviderConfig {
            name: "测试API".into(),
            enabled: true,
            url: url.into(),
            headers: BTreeMap::new(),
            json_pointer: pointer.into(),
            interval_minutes: 30,
        }
    }

    #[tokio::test]
    async fn rejects_private_url() {
        let config = provider("http://192.168.1.5/api", "");
        let error = fetch(&config).await.unwrap_err();
        assert!(error.contains("拒绝"));
    }

    #[tokio::test]
    async fn rejects_missing_url() {
        let config = provider("", "");
        assert!(fetch(&config).await.is_err());
    }
}
