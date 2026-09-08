//! 额度查询 Provider 集合与调度器。
//!
//! 所有 Provider 的出站请求统一走 [`crate::net::validate_public_url`] 安全校验
//! （仅 http/https；拒绝 localhost/环回/私有/保留地址——这些是公网数据源）。

pub mod custom;
pub mod deepseek;
pub mod glm;
pub mod siliconflow;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::Config;
use crate::ntfy::Notification;

/// 单个 Provider 的一次查询结果。
#[derive(Debug, Clone)]
pub struct QuotaResult {
    pub provider: String,
    /// 展示行（如 "GLM 5小时窗: 42%（重置于 18:30）"）
    pub lines: Vec<String>,
    /// 托盘 tooltip 用的一行紧凑摘要（如 "42%/67%"、"¥110.30"）
    pub short: String,
    /// 告警等级（用量百分比；余额类 Provider 为 None）
    pub warn_percent: Option<u64>,
}

impl QuotaResult {
    pub fn display_summary(&self) -> String {
        self.lines.join("\n")
    }
}

/// 托盘 tooltip 的额度看板：各 Provider 最新状态。
#[derive(Debug, Default, Clone)]
pub struct QuotaBoard {
    /// 顺序与 config 中 Provider 顺序一致
    pub entries: Vec<ProviderStatus>,
}

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub name: String,
    /// 紧凑摘要；失败时为错误提示
    pub short: String,
    pub failed: bool,
}

impl QuotaBoard {
    /// 生成 tooltip 摘要文本："GLM 42%/67% · DeepSeek ¥110.23"。
    /// Windows tooltip 上限 128 字符，超长截断。
    pub fn tooltip_summary(&self) -> String {
        if self.entries.is_empty() {
            return String::from("额度未配置");
        }
        let parts: Vec<String> = self
            .entries
            .iter()
            .map(|entry| {
                if entry.failed {
                    format!("{} ✗", entry.name)
                } else {
                    format!("{} {}", entry.name, entry.short)
                }
            })
            .collect();
        let text = parts.join(" · ");
        if text.chars().count() > 100 {
            format!("{}…", text.chars().take(100).collect::<String>())
        } else {
            text
        }
    }
}

/// Provider 种类（调度任务用）
#[derive(Debug, Clone)]
pub enum ProviderKind {
    Glm,
    DeepSeek,
    SiliconFlow,
    Custom(usize),
}

/// 查询单个 Provider
pub async fn fetch_kind(config: &Config, kind: &ProviderKind) -> Result<QuotaResult, String> {
    match kind {
        ProviderKind::Glm => glm::fetch(&config.glm).await,
        ProviderKind::DeepSeek => deepseek::fetch(&config.deepseek).await,
        ProviderKind::SiliconFlow => siliconflow::fetch(&config.siliconflow).await,
        ProviderKind::Custom(index) => {
            let provider = config
                .custom
                .get(*index)
                .ok_or_else(|| format!("custom[{index}] 配置不存在"))?;
            custom::fetch(provider).await
        }
    }
}

/// 查询所有启用的 Provider（query 命令与托盘菜单共用）。
pub async fn query_all(config: &Config) -> Vec<(ProviderKind, Result<QuotaResult, String>)> {
    let mut results = Vec::new();
    if config.glm.enabled {
        results.push((ProviderKind::Glm, glm::fetch(&config.glm).await));
    }
    if config.deepseek.enabled {
        results.push((
            ProviderKind::DeepSeek,
            deepseek::fetch(&config.deepseek).await,
        ));
    }
    if config.siliconflow.enabled {
        results.push((
            ProviderKind::SiliconFlow,
            siliconflow::fetch(&config.siliconflow).await,
        ));
    }
    for index in 0..config.custom.len() {
        if config.custom[index].enabled {
            results.push((
                ProviderKind::Custom(index),
                custom::fetch(&config.custom[index]).await,
            ));
        }
    }
    results
}

/// 把查询结果渲染成通知（成功行 + 失败行都包含，便于远程诊断）。
pub fn render_notification(results: &[(ProviderKind, Result<QuotaResult, String>)]) -> Notification {
    let mut body_lines = Vec::new();
    for (_, result) in results {
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

/// 用查询结果刷新看板
pub fn update_board(
    board: &Arc<Mutex<QuotaBoard>>,
    results: &[(ProviderKind, Result<QuotaResult, String>)],
) {
    let mut guard = board.lock().unwrap();
    for (kind, result) in results {
        let (name, short, failed) = match result {
            Ok(item) => (item.provider.clone(), item.short.clone(), false),
            Err(_) => (kind_name(kind), "查询失败".to_string(), true),
        };
        upsert_entry(&mut guard, name, short, failed);
    }
}

fn kind_name(kind: &ProviderKind) -> String {
    match kind {
        ProviderKind::Glm => "GLM".to_string(),
        ProviderKind::DeepSeek => "DeepSeek".to_string(),
        ProviderKind::SiliconFlow => "SiliconFlow".to_string(),
        ProviderKind::Custom(index) => format!("自定义{index}"),
    }
}

fn upsert_entry(board: &mut QuotaBoard, name: String, short: String, failed: bool) {
    match board.entries.iter_mut().find(|entry| entry.name == name) {
        Some(entry) => {
            entry.short = short;
            entry.failed = failed;
        }
        None => board.entries.push(ProviderStatus { name, short, failed }),
    }
}

/// 统一 HTTP 客户端（带超时）
pub(crate) fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

struct SchedulerTask {
    kind: ProviderKind,
    name: String,
    interval_secs: u64,
    threshold: u64,
}

/// 调度器：每个 Provider 独立任务、独立间隔；失败指数退避（×2，封顶 8 倍）；
/// 用量跨过阈值时推送一次告警，回落或窗口重置后重新武装。
/// 看板实时更新，托盘 tooltip 读取。
pub async fn run_scheduler(
    config: Config,
    board: Arc<Mutex<QuotaBoard>>,
    sender: tokio::sync::mpsc::Sender<Notification>,
) {
    for task in build_tasks(&config) {
        let board = board.clone();
        let sender = sender.clone();
        let config = config.clone();
        tokio::spawn(async move {
            provider_loop(config, task, board, sender).await;
        });
    }
}

fn build_tasks(config: &Config) -> Vec<SchedulerTask> {
    let mut tasks = Vec::new();
    if config.glm.enabled && !config.glm.api_key.trim().is_empty() {
        tasks.push(SchedulerTask {
            kind: ProviderKind::Glm,
            name: "GLM".to_string(),
            interval_secs: config.glm.interval_minutes.max(1) * 60,
            threshold: config.glm.warn_threshold_percent,
        });
    }
    if config.deepseek.enabled && !config.deepseek.api_key.trim().is_empty() {
        tasks.push(SchedulerTask {
            kind: ProviderKind::DeepSeek,
            name: "DeepSeek".to_string(),
            interval_secs: config.deepseek.interval_minutes.max(1) * 60,
            threshold: 80,
        });
    }
    if config.siliconflow.enabled && !config.siliconflow.api_key.trim().is_empty() {
        tasks.push(SchedulerTask {
            kind: ProviderKind::SiliconFlow,
            name: "SiliconFlow".to_string(),
            interval_secs: config.siliconflow.interval_minutes.max(1) * 60,
            threshold: 80,
        });
    }
    for (index, provider) in config.custom.iter().enumerate() {
        if provider.enabled && !provider.url.trim().is_empty() {
            tasks.push(SchedulerTask {
                kind: ProviderKind::Custom(index),
                name: if provider.name.trim().is_empty() {
                    format!("自定义{index}")
                } else {
                    provider.name.trim().to_string()
                },
                interval_secs: provider.interval_minutes.max(1) * 60,
                threshold: 80,
            });
        }
    }
    tasks
}

async fn provider_loop(
    config: Config,
    task: SchedulerTask,
    board: Arc<Mutex<QuotaBoard>>,
    sender: tokio::sync::mpsc::Sender<Notification>,
) {
    let mut backoff_multiplier: u64 = 1;
    let mut warn_armed = true;
    loop {
        match fetch_kind(&config, &task.kind).await {
            Ok(result) => {
                backoff_multiplier = 1;
                upsert_entry(&mut board.lock().unwrap(), task.name.clone(), result.short.clone(), false);
                if let Some(percent) = result.warn_percent {
                    if percent >= task.threshold {
                        if warn_armed {
                            warn_armed = false;
                            let _ = sender
                                .send(Notification {
                                    title: "额度告警".to_string(),
                                    body: result.display_summary(),
                                    tags: vec!["warning".to_string()],
                                    priority: 5,
                                })
                                .await;
                        }
                    } else {
                        warn_armed = true;
                    }
                }
            }
            Err(error) => {
                upsert_entry(&mut board.lock().unwrap(), task.name.clone(), "查询失败".to_string(), true);
                eprintln!("[额度查询] {} 失败: {error}", task.name);
                backoff_multiplier = next_backoff(backoff_multiplier);
            }
        }
        tokio::time::sleep(Duration::from_secs(task.interval_secs * backoff_multiplier)).await;
    }
}

/// 指数退避：1 → 2 → 4 → 8（封顶）
fn next_backoff(current: u64) -> u64 {
    (current * 2).min(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_result_summary_joins_lines() {
        let result = QuotaResult {
            provider: "glm".into(),
            lines: vec!["5小时窗: 40%".into(), "周窗: 12%".into()],
            short: "40%/12%".into(),
            warn_percent: Some(40),
        };
        assert_eq!(result.display_summary(), "5小时窗: 40%\n周窗: 12%");
    }

    #[test]
    fn notification_renders_success_and_failure() {
        let results = vec![
            (
                ProviderKind::Glm,
                Ok(QuotaResult {
                    provider: "glm".into(),
                    lines: vec!["5小时窗: 40%".into()],
                    short: "40%".into(),
                    warn_percent: Some(40),
                }),
            ),
            (ProviderKind::DeepSeek, Err("DeepSeek 查询失败".to_string())),
        ];
        let notification = render_notification(&results);
        assert!(notification.body.contains("5小时窗"));
        assert!(notification.body.contains("DeepSeek 查询失败"));
    }

    #[test]
    fn board_tooltip_joins_and_truncates() {
        let mut board = QuotaBoard::default();
        board.entries.push(ProviderStatus {
            name: "GLM".into(),
            short: "42%/67%".into(),
            failed: false,
        });
        board.entries.push(ProviderStatus {
            name: "DeepSeek".into(),
            short: "¥110.23".into(),
            failed: false,
        });
        assert_eq!(board.tooltip_summary(), "GLM 42%/67% · DeepSeek ¥110.23");

        board.entries[1].failed = true;
        assert!(board.tooltip_summary().contains("DeepSeek ✗"));

        board.entries.push(ProviderStatus {
            name: "X".repeat(200),
            short: String::new(),
            failed: false,
        });
        assert!(board.tooltip_summary().chars().count() <= 101);
    }

    #[test]
    fn backoff_doubles_and_caps_at_8() {
        assert_eq!(next_backoff(1), 2);
        assert_eq!(next_backoff(2), 4);
        assert_eq!(next_backoff(4), 8);
        assert_eq!(next_backoff(8), 8);
    }

    #[test]
    fn build_tasks_skips_empty_keys_and_disabled() {
        let mut config = Config::default();
        config.glm.api_key = "k".into();
        config.deepseek.enabled = true; // 无 key
        config.siliconflow.enabled = true;
        config.siliconflow.api_key = "k".into();
        let tasks = build_tasks(&config);
        let names: Vec<&str> = tasks.iter().map(|task| task.name.as_str()).collect();
        assert_eq!(names, vec!["GLM", "SiliconFlow"]);
    }

    #[test]
    fn upsert_updates_existing_entry() {
        let mut board = QuotaBoard::default();
        upsert_entry(&mut board, "GLM".into(), "10%".into(), false);
        upsert_entry(&mut board, "GLM".into(), "20%".into(), false);
        assert_eq!(board.entries.len(), 1);
        assert_eq!(board.entries[0].short, "20%");
    }
}
