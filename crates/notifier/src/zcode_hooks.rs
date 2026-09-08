//! zcode hooks 安装/卸载：读写合并 `~/.zcode/cli/config.json`。
//!
//! 采用参考项目的 owner-marker 模式：写入的 hook 命令内嵌 `--owner miband-notifier`
//! 标记，重装/卸载时只增删自己的条目，绝不触碰用户已有的 hooks、plugins 等配置；
//! 写入前生成 `.bak` 备份；结构为 zcode 官方的 `hooks.events.<Event>` 形态，
//! 事件数组每个 matcher 分组内放一个 process 类型 hook（参数向量，不经 shell）。

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::hook::HookEvent;

/// 安装到 hook 命令里的属主标记（卸载时靠它识别自己的条目）
pub const OWNER_MARKER: &str = "--owner miband-notifier";

/// 事件 → matcher 分组键（本项目两个事件都不需要 matcher，匹配所有工具）
const EVENTS: [HookEvent; 2] = [HookEvent::Stop, HookEvent::PermissionRequest];

pub fn zcode_config_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| "无法定位用户目录（USERPROFILE 未设置）".to_string())?;
    let path = home.join(".zcode").join("cli").join("config.json");
    Ok(path)
}

/// 安装（幂等）：先移除旧的自有条目，再追加新条目。
pub fn install(exe_path: &str) -> Result<String, String> {
    let config_path = zcode_config_path()?;
    let existing = fs::read_to_string(&config_path).unwrap_or_else(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            String::new()
        } else {
            return format!("读取 {} 失败: {error}", config_path.display());
        }
    });
    let mut root = parse_config(&existing)?;
    backup(&config_path, &existing)?;
    remove_owned(&mut root)?;

    // hooks.enabled 必须显式开启（配置文件 hooks 默认禁用）
    let hooks = root
        .as_object_mut()
        .expect("validated root")
        .entry("hooks")
        .or_insert_with(|| json!({}));
    let hooks_object = hooks
        .as_object_mut()
        .ok_or_else(|| "config.json 的 hooks 字段必须是对象".to_string())?;
    hooks_object.insert("enabled".to_string(), json!(true));

    let events = hooks_object
        .entry("events")
        .or_insert_with(|| json!({}));
    let events_object = events
        .as_object_mut()
        .ok_or_else(|| "config.json 的 hooks.events 字段必须是对象".to_string())?;

    for event in EVENTS {
        let event_name = event_config_name(event);
        let groups = events_object
            .entry(event_name)
            .or_insert_with(|| json!([]));
        let groups_array = groups
            .as_array_mut()
            .ok_or_else(|| format!("hooks.events.{event_name} 必须是数组"))?;
        groups_array.push(owned_group(exe_path, event));
    }

    write_config(&config_path, &root)?;
    Ok(format!("已安装 zcode hooks（{}）", config_path.display()))
}

/// 卸载：仅移除带属主标记的条目。
pub fn uninstall() -> Result<String, String> {
    let config_path = zcode_config_path()?;
    let existing = fs::read_to_string(&config_path)
        .map_err(|error| format!("读取 {} 失败: {error}", config_path.display()))?;
    let mut root = parse_config(&existing)?;
    backup(&config_path, &existing)?;
    remove_owned(&mut root)?;
    write_config(&config_path, &root)?;
    Ok(format!("已卸载 zcode hooks（{}）", config_path.display()))
}

fn event_config_name(event: HookEvent) -> &'static str {
    match event {
        HookEvent::Stop => "Stop",
        HookEvent::PermissionRequest => "PermissionRequest",
    }
}

/// 生成本项目专属的 hook 分组（process 类型 + 参数向量 + 3 秒超时）。
fn owned_group(exe_path: &str, event: HookEvent) -> Value {
    json!({
        "hooks": [{
            "type": "process",
            "command": exe_path,
            "args": [
                "--hook",
                event.as_str(),
                OWNER_MARKER
            ],
            "timeoutMs": 3000,
            "statusMessage": "MiBandNotifier 事件转发"
        }]
    })
}

fn parse_config(existing: &str) -> Result<Value, String> {
    if existing.trim().is_empty() {
        return Ok(json!({}));
    }
    let value: Value = serde_json::from_str(existing)
        .map_err(|_| "config.json 不是合法 JSON".to_string())?;
    if !value.is_object() {
        return Err("config.json 根必须是对象".to_string());
    }
    Ok(value)
}

fn backup(config_path: &PathBuf, contents: &str) -> Result<(), String> {
    if contents.trim().is_empty() {
        return Ok(());
    }
    let backup_path = config_path.with_extension("json.bak");
    fs::write(&backup_path, contents)
        .map_err(|error| format!("写备份失败 {}: {error}", backup_path.display()))
}

fn write_config(config_path: &PathBuf, root: &Value) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(root)
        .map_err(|_| "config.json 序列化失败".to_string())?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建目录失败 {} : {error}", parent.display()))?;
    }
    fs::write(config_path, serialized)
        .map_err(|error| format!("写入 {} 失败: {error}", config_path.display()))
}

/// 移除所有带属主标记的 hook 条目；空分组一并删除。
fn remove_owned(root: &mut Value) -> Result<(), String> {
    let root_object = root
        .as_object_mut()
        .ok_or_else(|| "config.json 根必须是对象".to_string())?;
    let Some(hooks) = root_object.get_mut("hooks") else {
        return Ok(());
    };
    let hooks_object = hooks
        .as_object_mut()
        .ok_or_else(|| "config.json 的 hooks 字段必须是对象".to_string())?;
    let Some(events) = hooks_object.get_mut("events") else {
        return Ok(());
    };
    let events_object = events
        .as_object_mut()
        .ok_or_else(|| "config.json 的 hooks.events 字段必须是对象".to_string())?;
    let event_names: Vec<String> = events_object.keys().cloned().collect();
    for event_name in event_names {
        let Some(groups) = events_object.get_mut(&event_name).and_then(Value::as_array_mut)
        else {
            continue;
        };
        groups.retain_mut(|group| {
            let Some(handlers) = group
                .as_object_mut()
                .and_then(|object| object.get_mut("hooks"))
                .and_then(Value::as_array_mut)
            else {
                return true;
            };
            handlers.retain(|handler| !is_owned_handler(handler));
            !handlers.is_empty()
        });
        if groups.is_empty() {
            events_object.remove(&event_name);
        }
    }
    // events 为空则整个移除，保持配置整洁
    if events_object.is_empty() {
        hooks_object.remove("events");
    }
    Ok(())
}

fn is_owned_handler(handler: &Value) -> bool {
    handler
        .as_object()
        .into_iter()
        .flat_map(|object| {
            object
                .get("args")
                .and_then(Value::as_array)
                .map(|args| args.iter().filter_map(Value::as_str))
                .into_iter()
                .flatten()
                .chain(object.get("command").and_then(Value::as_str))
        })
        .any(|piece| piece.contains(OWNER_MARKER))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_into_empty_config() {
        let mut root = parse_config("").unwrap();
        remove_owned(&mut root).unwrap();
        let object = root.as_object_mut().unwrap();
        let hooks = object.entry("hooks").or_insert_with(|| json!({}));
        let hooks_object = hooks.as_object_mut().unwrap();
        hooks_object.insert("enabled".into(), json!(true));
        let events = hooks_object.entry("events").or_insert_with(|| json!({}));
        let events_object = events.as_object_mut().unwrap();
        for event in EVENTS {
            let groups = events_object.entry(event_config_name(event)).or_insert_with(|| json!([]));
            groups.as_array_mut().unwrap().push(owned_group("C:\\app.exe", event));
        }
        assert!(root["hooks"]["enabled"].as_bool().unwrap());
        let groups = root["hooks"]["events"]["Stop"].as_array().unwrap();
        assert_eq!(groups.len(), 1);
        let args = groups[0]["hooks"][0]["args"].as_array().unwrap();
        assert!(args.iter().any(|a| a.as_str() == Some(OWNER_MARKER)));
    }

    #[test]
    fn reinstall_replaces_owned_and_keeps_foreign() {
        // 用户已有 hooks（含无关条目），重装后无关条目保留、自有条目唯一
        let mut foreign_root: Value = serde_json::from_str(
            r#"{"plugins":{"enabledPlugins":{}},"hooks":{"enabled":true,"events":{"Stop":[{"hooks":[{"type":"command","command":"echo other"}]}]}}}"#,
        )
        .unwrap();
        remove_owned(&mut foreign_root).unwrap();
        let object = foreign_root.as_object_mut().unwrap();
        let hooks = object.entry("hooks").or_insert_with(|| json!({}));
        let hooks_object = hooks.as_object_mut().unwrap();
        hooks_object.insert("enabled".into(), json!(true));
        let events = hooks_object.entry("events").or_insert_with(|| json!({}));
        let events_object = events.as_object_mut().unwrap();
        for event in EVENTS {
            let groups = events_object.entry(event_config_name(event)).or_insert_with(|| json!([]));
            groups.as_array_mut().unwrap().push(owned_group("C:\\app.exe", event));
        }
        // 模拟重装：再跑一次 remove_owned + 追加
        remove_owned(&mut foreign_root).unwrap();
        let hooks_object = foreign_root["hooks"].as_object_mut().unwrap();
        let events_object = hooks_object.get_mut("events").unwrap().as_object_mut().unwrap();
        for event in EVENTS {
            let groups = events_object.entry(event_config_name(event)).or_insert_with(|| json!([]));
            groups.as_array_mut().unwrap().push(owned_group("C:\\app.exe", event));
        }
        let stop_groups = foreign_root["hooks"]["events"]["Stop"].as_array().unwrap();
        assert_eq!(stop_groups.len(), 2);
        assert!(stop_groups
            .iter()
            .any(|g| g["hooks"][0]["command"].as_str() == Some("echo other")));
        assert!(foreign_root["plugins"]["enabledPlugins"].is_object());
    }

    #[test]
    fn uninstall_removes_only_owned() {
        let config = r#"{
            "hooks": {
                "enabled": true,
                "events": {
                    "Stop": [
                        {"hooks": [{"type":"command","command":"echo other"}]},
                        {"hooks": [{"type":"process","command":"C:\\app.exe","args":["--hook","stop","--owner miband-notifier"]}]}
                    ]
                }
            }
        }"#;
        let mut root: Value = serde_json::from_str(config).unwrap();
        remove_owned(&mut root).unwrap();
        let stop_groups = root["hooks"]["events"]["Stop"].as_array().unwrap();
        assert_eq!(stop_groups.len(), 1);
        assert_eq!(stop_groups[0]["hooks"][0]["command"], json!("echo other"));
    }

    #[test]
    fn owned_handler_detects_marker_in_args() {
        let handler = json!({
            "type": "process",
            "command": "C:\\app.exe",
            "args": ["--hook", "stop", OWNER_MARKER]
        });
        assert!(is_owned_handler(&handler));
        let foreign = json!({"type": "command", "command": "echo hi"});
        assert!(!is_owned_handler(&foreign));
    }
}
