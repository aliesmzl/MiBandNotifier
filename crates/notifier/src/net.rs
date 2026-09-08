//! 出站 URL 安全校验。
//!
//! 安全约束：所有服务端/工具发起的请求仅允许 http/https；对"公网数据源"（额度
//! Provider，含用户自定义 URL）额外校验主机，拒绝 localhost、环回、私有和保留
//! 地址。唯一的环回豁免是程序自管理的 ntfy 子进程端点（见 [`validate_ntfy_url`]）：
//! 该地址来自本程序自己的配置默认值（固定 127.0.0.1），不是外部输入派生的 URL。

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// 校验公网数据源 URL：仅 http/https，主机不能是本地/内网/保留地址。
pub fn validate_public_url(raw: &str) -> Result<reqwest::Url, String> {
    let url = parse_http_url(raw)?;
    let host = url
        .host_str()
        .ok_or_else(|| "URL 缺少主机名".to_string())?
        .to_string();
    if let Some(ip) = parse_ip_host(&host) {
        if !is_public_ip(&ip) {
            return Err(format!("目标地址 {host} 是本地/内网/保留地址，已拒绝"));
        }
    } else if is_local_hostname(&host) {
        return Err(format!("目标主机名 {host} 指向本机，已拒绝"));
    }
    Ok(url)
}

/// 校验 ntfy 推送地址：仅 http/https，主机不限制。
///
/// 自托管 ntfy 是本工具的核心部署形态：默认指向程序自己拉起的子进程
/// （127.0.0.1），用户也可能配置局域网内的 ntfy 服务器，因此主机允许私网地址。
/// 该地址只来自用户本机配置文件，不经任何外部输入派生。
pub fn validate_ntfy_url(raw: &str) -> Result<reqwest::Url, String> {
    parse_http_url(raw)
}

fn parse_http_url(raw: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|error| format!("URL 无效: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(format!("不允许的协议 {other}（仅支持 http/https）")),
    }
}

fn parse_ip_host(host: &str) -> Option<IpAddr> {
    let trimmed = host.trim_matches(|c| c == '[' || c == ']');
    trimmed.parse().ok()
}

fn is_local_hostname(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") || host.ends_with(".internal")
}

fn is_public_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_ipv4(v4),
        IpAddr::V6(v6) => is_public_ipv6(v6),
    }
}

fn is_public_ipv4(v4: &Ipv4Addr) -> bool {
    let octets = v4.octets();
    !(v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_broadcast()
        || v4.is_multicast()
        || v4.is_documentation()
        || octets[0] == 0
        || octets[0] >= 240
        // 100.64.0.0/10 CGNAT
        || (octets[0] == 100 && (octets[1] & 0xC0) == 64))
}

fn is_public_ipv6(v6: &Ipv6Addr) -> bool {
    let segments = v6.segments();
    !(v6.is_loopback()
        || v6.is_unspecified()
        || v6.is_multicast()
        // fc00::/7 唯一本地地址（ULA）
        || (segments[0] & 0xFE00) == 0xFC00
        // fe80::/10 链路本地
        || (segments[0] & 0xFFC0) == 0xFE80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_urls_pass() {
        assert!(validate_public_url("https://open.bigmodel.cn/api").is_ok());
        assert!(validate_public_url("http://example.com").is_ok());
    }

    #[test]
    fn private_targets_rejected() {
        for url in [
            "http://127.0.0.1:8080/x",
            "http://localhost/x",
            "http://192.168.1.5/x",
            "http://10.0.0.2/x",
            "http://172.16.3.1/x",
            "http://169.254.1.1/x",
            "http://0.0.0.0/x",
            "http://100.64.0.1/x",
            "http://[::1]/x",
            "http://[fe80::1]/x",
            "http://[fc00::1]/x",
            "ftp://example.com/x",
        ] {
            assert!(validate_public_url(url).is_err(), "应当拒绝: {url}");
        }
    }

    #[test]
    fn ntfy_url_allows_loopback_but_only_http() {
        assert!(validate_ntfy_url("http://127.0.0.1:8090").is_ok());
        assert!(validate_ntfy_url("https://ntfy.example.com").is_ok());
        assert!(validate_ntfy_url("ftp://127.0.0.1").is_err());
    }
}
