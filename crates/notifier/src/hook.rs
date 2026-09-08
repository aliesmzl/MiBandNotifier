//! hook 事件信箱（spool 目录）。
//!
//! zcode hook 以 `MiBandNotifier.exe --hook <event>` 短进程运行（3 秒超时），
//! 它只把 stdin 的 JSON 原样落盘到 spool 目录后立即退出——常驻进程不在线也
//! 不阻塞、不丢事件（下次启动补推）；常驻进程定期轮询消费并删除文件。
//!
//! 文件名含纳秒时间戳 + 进程 ID + 自增计数，字典序即时间序；写入用
//! 临时文件 + rename 原子提交。文件大小上限 32 KiB，超限只保留截断后的
//! 头部（hook 载荷只用于展示摘要，无需完整内容）。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 单个事件文件大小上限
const MAX_EVENT_BYTES: usize = 32 * 1024;
/// spool 目录内文件总数上限（防御性：避免异常情况下目录膨胀）
const MAX_SPOOL_FILES: usize = 1000;

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 支持的 hook 事件（zcode 七事件中本项目订阅的子集）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    Stop,
    PermissionRequest,
}

impl HookEvent {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "stop" | "Stop" => Some(Self::Stop),
            "permission" | "permission_request" | "PermissionRequest" => {
                Some(Self::PermissionRequest)
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::PermissionRequest => "permission_request",
        }
    }

    /// 通知展示标题
    pub fn display_title(&self) -> &'static str {
        match self {
            Self::Stop => "任务完成",
            Self::PermissionRequest => "等待授权",
        }
    }
}

/// spool 中一条事件的信封。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoolRecord {
    pub event: HookEvent,
    pub observed_at_ms: i64,
    /// zcode 传入的原始 hook JSON（可能被截断）
    pub payload: String,
}

impl SpoolRecord {
    /// 从 hook 载荷提取一行摘要（用于通知正文）。
    ///
    /// zcode Stop 事件的载荷含 `stop_hook_response`/`last_response` 等字段；
    /// PermissionRequest 含工具名等。这里做通用提取：优先取常见摘要字段，
    /// 否则取首个非空字符串值，最后回退固定文案。
    pub fn summary_line(&self) -> String {
        let value: serde_json::Value = match serde_json::from_str(&self.payload) {
            Ok(value) => value,
            Err(_) => return String::new(),
        };
        for key in ["summary", "response_preview", "last_response", "prompt", "tool_name", "tool"] {
            if let Some(found) = extract_short_text(&value, key) {
                return found;
            }
        }
        // 兜底：任意一级字符串字段
        if let Some(object) = value.as_object() {
            for (key, item) in object {
                if key == "session_id" {
                    continue;
                }
                if let Some(text) = item.as_str() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return truncate_chars(trimmed, 60);
                    }
                }
            }
        }
        String::new()
    }
}

fn extract_short_text(value: &serde_json::Value, key: &str) -> Option<String> {
    let text = value.get(key)?.as_str()?.trim();
    if text.is_empty() {
        return None;
    }
    Some(truncate_chars(text, 60))
}

/// 按字符截断（中文安全），加省略号。
fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{truncated}…")
}

/// hook 短进程入口：读 stdin JSON，落盘后立即返回。
pub fn enqueue_hook_event(
    spool_dir: &Path,
    event: HookEvent,
    payload: &str,
) -> Result<(), String> {
    let record = SpoolRecord {
        event,
        observed_at_ms: chrono::Utc::now().timestamp_millis(),
        payload: truncate_bytes(payload, MAX_EVENT_BYTES),
    };
    let serialized = serde_json::to_vec(&record).map_err(|error| error.to_string())?;
    fs::create_dir_all(spool_dir).map_err(|error| format!("创建 spool 目录失败: {error}"))?;
    enforce_spool_limit(spool_dir)?;
    let stem = event_stem();
    let temporary = spool_dir.join(format!(".{stem}.tmp"));
    let destination = spool_dir.join(format!("{stem}.json"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("创建临时文件失败: {error}"))?;
    file.write_all(&serialized)
        .map_err(|error| format!("写入事件失败: {error}"))?;
    file.sync_all().ok();
    drop(file);
    fs::rename(&temporary, &destination)
        .map_err(|error| format!("提交事件失败: {error}"))?;
    Ok(())
}

/// 常驻进程：取出并删除 spool 中所有待处理事件（按时间序）。
pub fn drain_events(spool_dir: &Path) -> Vec<SpoolRecord> {
    let entries = match fs::read_dir(spool_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let parsed = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SpoolRecord>(&bytes).ok());
        let _ = fs::remove_file(&path);
        if let Some(record) = parsed {
            records.push(record);
        }
    }
    records
}

/// 清理过旧事件（常驻进程启动时调用；默认保留 7 天）。
pub fn prune_stale_events(spool_dir: &Path, max_age: chrono::Duration) {
    let cutoff_ms = chrono::Utc::now().timestamp_millis() - max_age.num_milliseconds();
    let entries = match fs::read_dir(spool_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let stale = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SpoolRecord>(&bytes).ok())
            .map(|record| record.observed_at_ms < cutoff_ms)
            .unwrap_or(true);
        if stale {
            let _ = fs::remove_file(&path);
        }
    }
}

fn enforce_spool_limit(spool_dir: &Path) -> Result<(), String> {
    let entries = match fs::read_dir(spool_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
        .collect();
    if paths.len() < MAX_SPOOL_FILES {
        return Ok(());
    }
    paths.sort();
    let excess = paths.len() + 1 - MAX_SPOOL_FILES;
    for path in paths.into_iter().take(excess) {
        let _ = fs::remove_file(&path);
    }
    Ok(())
}

fn event_stem() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp:032x}-{:08x}", std::process::id() as u32 & 0xFFFF_FFFF)
        + &format!("-{counter:016x}")
}

/// 按字节截断到 UTF-8 边界。
fn truncate_bytes(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mbn-spool-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn hook_event_parses_aliases() {
        assert_eq!(HookEvent::parse("stop"), Some(HookEvent::Stop));
        assert_eq!(HookEvent::parse("Stop"), Some(HookEvent::Stop));
        assert_eq!(
            HookEvent::parse("permission"),
            Some(HookEvent::PermissionRequest)
        );
        assert_eq!(HookEvent::parse("unknown"), None);
    }

    #[test]
    fn enqueue_and_drain_roundtrip() {
        let dir = temp_dir();
        let payload = r#"{"session_id":"s1","tool_name":"Bash","summary":"构建完成"}"#;
        enqueue_hook_event(&dir, HookEvent::Stop, payload).unwrap();
        enqueue_hook_event(&dir, HookEvent::PermissionRequest, payload).unwrap();
        let records = drain_events(&dir);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].event, HookEvent::Stop);
        assert_eq!(records[1].event, HookEvent::PermissionRequest);
        assert_eq!(records[0].summary_line(), "构建完成");
        // drain 后目录清空
        assert!(drain_events(&dir).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversize_payload_is_truncated() {
        let dir = temp_dir();
        let large = "x".repeat(64 * 1024);
        assert!(enqueue_hook_event(&dir, HookEvent::Stop, &large).is_ok());
        let records = drain_events(&dir);
        assert_eq!(records.len(), 1);
        assert!(records[0].payload.len() <= 32 * 1024);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_events_are_pruned() {
        let dir = temp_dir();
        let old_record = SpoolRecord {
            event: HookEvent::Stop,
            observed_at_ms: 0,
            payload: "{}".to_string(),
        };
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("old.json"), serde_json::to_vec(&old_record).unwrap()).unwrap();
        prune_stale_events(&dir, chrono::Duration::days(7));
        assert!(drain_events(&dir).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn utf8_truncation_keeps_boundary() {
        let text = "你好世界".repeat(100);
        let truncated = truncate_bytes(&text, 100);
        assert!(truncated.len() <= 100);
        assert!(text.is_char_boundary(truncated.len()) || truncated.is_empty());
    }
}
