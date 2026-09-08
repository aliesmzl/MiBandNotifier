//! ntfy 推送客户端。
//!
//! 部署形态：ntfy.exe 以 Windows 服务方式常驻（见 scripts/setup-ntfy-service.ps1，
//! 监听 0.0.0.0:8090，供局域网手机订阅）。本程序只通过 HTTP 发布通知：
//! 手机 ntfy App 订阅 `http://<PC局域网IP>:8090/<topic>` 获得即时送达
//! （前台服务长连接，无需 GCM），再由"小米运动健康"的「应用通知提醒」
//! 镜像到小米手环 9 Pro。
//!
//! 推送地址只来自用户本机配置文件（默认 127.0.0.1 服务端点），
//! 且统一走 [`crate::net::validate_ntfy_url`] 校验，仅允许 http/https。

use std::time::Duration;

use crate::config::NtfyConfig;
use crate::net;

/// 一次 ntfy 推送。
pub struct Notification {
    pub title: String,
    pub body: String,
    /// ntfy Tags，可映射手机端图标；默认空
    pub tags: Vec<String>,
    /// 通知优先级（1-5）；提醒类事件建议 4（high）
    pub priority: u8,
}

pub struct NtfyClient {
    server_url: reqwest::Url,
    topic: String,
    http: reqwest::Client,
}

impl NtfyClient {
    pub fn new(config: &NtfyConfig) -> Result<Self, String> {
        let server_url = net::validate_ntfy_url(&config.server_url)?;
        if config.topic.trim().is_empty() {
            return Err("ntfy topic 为空，请先完成初始化配置".to_string());
        }
        Ok(Self {
            server_url,
            topic: config.topic.trim().to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))?,
        })
    }

    /// 探测服务健康状态（GET /v1/health）。
    pub async fn health(&self) -> Result<bool, String> {
        let url = self
            .server_url
            .join("v1/health")
            .map_err(|error| format!("构造健康检查 URL 失败: {error}"))?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|error| format!("ntfy 服务不可达: {error}"))?;
        Ok(response.status().is_success())
    }

    /// 发布通知。返回 Ok(true) 表示送达；错误携带服务端返回信息。
    pub async fn publish(&self, notification: &Notification) -> Result<bool, String> {
        let url = self
            .server_url
            .join(&self.topic)
            .map_err(|error| format!("构造推送 URL 失败: {error}"))?;
        let mut request = self.http.post(url).header("Title", &notification.title);
        if !notification.tags.is_empty() {
            request = request.header("Tags", notification.tags.join(","));
        }
        if (1..=5).contains(&notification.priority) {
            request = request.header("Priority", notification.priority.to_string());
        }
        let response = request
            .body(notification.body.clone())
            .send()
            .await
            .map_err(|error| format!("ntfy 推送失败: {error}"))?;
        let status = response.status();
        if status.is_success() {
            Ok(true)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(format!("ntfy 服务返回 {status}: {body}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NtfyConfig;

    /// 测试 harness 不经过 main：每个测试前安装 ring CryptoProvider
    fn ensure_crypto_provider() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    #[test]
    fn client_builds_from_config() {
        ensure_crypto_provider();
        let config = NtfyConfig {
            server_url: "http://127.0.0.1:8090".into(),
            topic: "mbn-abc123".into(),
        };
        let client = NtfyClient::new(&config).unwrap();
        assert_eq!(client.topic, "mbn-abc123");
    }

    #[test]
    fn client_rejects_empty_topic() {
        let config = NtfyConfig { topic: String::new(), ..NtfyConfig::default() };
        assert!(NtfyClient::new(&config).is_err());
    }

    #[test]
    fn client_rejects_non_http_scheme() {
        let config = NtfyConfig {
            server_url: "ftp://127.0.0.1:8090".into(),
            topic: "mbn-abc123".into(),
            ..NtfyConfig::default()
        };
        assert!(NtfyClient::new(&config).is_err());
    }
}
