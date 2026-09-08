//! 配置文件：`%APPDATA%\MiBandNotifier\config.toml`。
//!
//! API Key 只存于此文件（用户目录，不进 git、不进日志）。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const APP_DIR_NAME: &str = "MiBandNotifier";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Windows toast 系统通知开关
    pub toast: bool,
    /// 事件推送开关
    pub events: EventToggles,
    /// ntfy 推送设置
    pub ntfy: NtfyConfig,
    /// GLM Coding Plan 额度查询
    pub glm: GlmConfig,
    /// DeepSeek 余额查询
    pub deepseek: BalanceProviderConfig,
    /// SiliconFlow 余额查询
    pub siliconflow: BalanceProviderConfig,
    /// 自定义 Provider（第三方 API）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<CustomProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EventToggles {
    /// 任务完成（zcode Stop）
    pub on_stop: bool,
    /// 等待授权（zcode PermissionRequest）
    pub on_permission: bool,
    /// 额度告警
    pub on_quota_warn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NtfyConfig {
    /// 自托管 ntfy 服务地址。默认为本机 Windows 服务 127.0.0.1:8090
    /// （服务由 scripts/setup-ntfy-service.ps1 一次性安装，监听 0.0.0.0 供手机订阅）。
    pub server_url: String,
    /// 推送主题。留空时首次启动自动生成随机主题并写回配置。
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GlmConfig {
    pub enabled: bool,
    /// bigmodel 控制台生成的 API Key
    pub api_key: String,
    /// 查询间隔（分钟）
    pub interval_minutes: u64,
    /// 用量告警阈值（百分比）
    pub warn_threshold_percent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BalanceProviderConfig {
    pub enabled: bool,
    pub api_key: String,
    pub interval_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CustomProviderConfig {
    pub name: String,
    pub enabled: bool,
    /// 完整 URL（仅 http/https；主机不能是本地/内网/保留地址）
    pub url: String,
    /// 附加请求头（如 Authorization）
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty", default)]
    pub headers: std::collections::BTreeMap<String, String>,
    /// JSON 指针（RFC 6901）提取要展示的值，如 /data/balance
    pub json_pointer: String,
    pub interval_minutes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            toast: true,
            events: EventToggles::default(),
            ntfy: NtfyConfig::default(),
            glm: GlmConfig::default(),
            deepseek: BalanceProviderConfig::default(),
            siliconflow: BalanceProviderConfig::default(),
            custom: Vec::new(),
        }
    }
}

impl Default for EventToggles {
    fn default() -> Self {
        Self { on_stop: true, on_permission: true, on_quota_warn: true }
    }
}

impl Default for NtfyConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:8090".to_string(),
            topic: String::new(),
        }
    }
}

impl Default for GlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: String::new(),
            interval_minutes: 5,
            warn_threshold_percent: 80,
        }
    }
}

impl Default for BalanceProviderConfig {
    fn default() -> Self {
        Self { enabled: false, api_key: String::new(), interval_minutes: 30 }
    }
}

impl CustomProviderConfig {
    pub fn default_interval() -> u64 {
        30
    }
}

impl Default for CustomProviderConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: false,
            url: String::new(),
            headers: std::collections::BTreeMap::new(),
            json_pointer: String::new(),
            interval_minutes: Self::default_interval(),
        }
    }
}

/// 配置目录：%APPDATA%\MiBandNotifier
pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("USERPROFILE").unwrap_or_else(|| ".".into());
            PathBuf::from(home).join("AppData").join("Roaming")
        });
    base.join(APP_DIR_NAME)
}

pub fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE_NAME)
}

/// 运行时目录：spool、日志等。与配置目录相同。
pub fn data_dir() -> PathBuf {
    config_dir()
}

/// 生成可读随机 topic（与配置文件中的其他随机值一致使用 getrandom）。
pub fn generate_topic() -> String {
    let mut bytes = [0u8; 12];
    if getrandom::fill(&mut bytes).is_err() {
        // 极端情况下退化为时间戳，保证功能可用
        return format!("mbn-{}", chrono::Utc::now().timestamp());
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>().chars().take(12).fold(
        String::from("mbn"),
        |mut acc, c| {
            if acc.len() < 15 {
                acc.push(c);
            }
            acc
        },
    )
}

/// 加载配置；文件不存在时生成默认配置（含随机 topic）。
pub fn load_or_create() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() {
        let mut config = Config::default();
        config.ntfy.topic = generate_topic();
        save(&config)?;
        return Ok(config);
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("读取配置失败 {}: {error}", path.display()))?;
    let mut config: Config = toml::from_str(&contents)
        .map_err(|error| format!("解析配置失败 {}: {error}", path.display()))?;
    if config.ntfy.topic.trim().is_empty() {
        config.ntfy.topic = generate_topic();
        save(&config)?;
    }
    Ok(config)
}

pub fn save(config: &Config) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|error| format!("创建目录失败 {}: {error}", dir.display()))?;
    let contents = toml::to_string_pretty(config)
        .map_err(|error| format!("序列化配置失败: {error}"))?;
    fs::write(config_path(), contents).map_err(|error| format!("写入配置失败: {error}"))
}

/// ntfy.exe 服务监听端口（与 scripts/setup-ntfy-service.ps1 保持一致）。
pub const NTFY_PORT: u16 = 8090;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_through_toml() {
        let config = Config::default();
        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.ntfy.server_url, config.ntfy.server_url);
        assert_eq!(parsed.glm.interval_minutes, 5);
        assert!(parsed.events.on_stop);
    }

    #[test]
    fn generated_topics_are_random_and_prefixed() {
        let first = generate_topic();
        let second = generate_topic();
        assert!(first.starts_with("mbn"));
        assert_ne!(first, second);
    }
}
