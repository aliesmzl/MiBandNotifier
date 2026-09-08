//! MiBandNotifier 主程序。
//!
//! 架构：
//! - zcode hook → `MiBandNotifier.exe --hook <event> --owner miband-notifier`
//!   短进程：读 stdin JSON 落盘 spool 后立即退出（不阻塞 zcode）
//! - 常驻托盘模式（无参数）：Win32 托盘图标 + 后台 tokio 任务轮询 spool，
//!   toast + ntfy 双通道推送
//! - CLI 子命令：install-hooks / uninstall-hooks / query / notify test /
//!   ntfy-info / --hook
//!
//! Win32 托盘采用参考项目 codex-quota-band 的模式：主线程跑消息泵，
//! tokio Runtime 放后台线程，两者通过静态 APP 状态交互。

#![cfg_attr(not(windows), allow(unused))]

mod config;
mod hook;
mod net;
mod ntfy;
mod quota;
mod toast;
mod zcode_hooks;

use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, WM_COMMAND, IDC_ARROW, MSG, WNDCLASSEXW, WM_APP,
    WM_CONTEXTMENU, WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WS_OVERLAPPED,
};

const APP_NAME: &str = "MiBandNotifier";
const TRAY_MESSAGE: u32 = WM_APP + 1;
const TRAY_ICON_ID: u32 = 1;

/// 共享后台服务：托盘菜单动作与退出时使用
struct AppService {
    config: config::Config,
    pause_push: std::sync::atomic::AtomicBool,
    quota_summary: Arc<Mutex<String>>,
    notification_tx: tokio::sync::mpsc::Sender<ntfy::Notification>,
    runtime: tokio::runtime::Runtime,
}

static APP: OnceLock<Arc<AppService>> = OnceLock::new();

fn main() {
    // reqwest 用 no-provider TLS 特性：进程启动时安装 ring CryptoProvider
    let _ = rustls::crypto::ring::default_provider().install_default();

    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--hook") {
        run_hook_forwarder(&arguments);
        return;
    }
    match arguments.get(1).map(String::as_str) {
        Some("install-hooks") => exit_with(zcode_hooks::install(&current_exe_string())),
        Some("uninstall-hooks") => exit_with(zcode_hooks::uninstall()),
        Some("query") => exit_with(run_query_cli()),
        Some("notify") => exit_with(run_notify_cli(&arguments[2..])),
        Some("ntfy-info") => exit_with(run_ntfy_info()),
        Some("help") | Some("--help") | Some("-h") => print_help_and_exit(),
        Some("tray") | None => {
            if let Err(error) = run_tray() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        Some(other) => {
            eprintln!("未知命令: {other}（用 help 查看用法）");
            std::process::exit(2);
        }
    }
}

fn current_exe_string() -> String {
    std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "MiBandNotifier.exe".to_string())
}

fn exit_with(result: Result<String, String>) {
    match result {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn print_help_and_exit() {
    println!(
        "{APP_NAME} — 把 zcode 任务事件与 AI 额度推送到小米手环\n\
         \n\
         用法:\n\
         \x20 MiBandNotifier                 常驻托盘模式（日常使用，开机自启）\n\
         \x20 MiBandNotifier install-hooks   安装 zcode hooks（写 ~/.zcode/cli/config.json）\n\
         \x20 MiBandNotifier uninstall-hooks 卸载 zcode hooks\n\
         \x20 MiBandNotifier query           立即查询额度并推送\n\
         \x20 MiBandNotifier notify test     发送测试通知（验收链路）\n\
         \x20 MiBandNotifier ntfy-info       显示手机订阅地址与防火墙提示\n\
         \x20 MiBandNotifier --hook <event> --owner miband-notifier   zcode hook 内部入口"
    );
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// --hook：短进程转发
// ---------------------------------------------------------------------------

fn run_hook_forwarder(arguments: &[String]) {
    let event_name = arguments
        .iter()
        .position(|argument| argument == "--hook")
        .and_then(|index| arguments.get(index + 1))
        .map(String::as_str)
        .unwrap_or("");
    let Some(event) = hook::HookEvent::parse(event_name) else {
        // 未知事件：静默成功，绝不阻塞 zcode
        std::process::exit(0);
    };
    let mut payload = String::new();
    let _ = std::io::stdin().read_to_string(&mut payload);
    let spool_dir = config::data_dir().join("hook-spool");
    if let Err(error) = hook::enqueue_hook_event(&spool_dir, event, &payload) {
        // 落盘失败仅写日志文件，不报错退出
        let _ = std::fs::write(config::data_dir().join("hook-forwarder-error.log"), &error);
    }
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// CLI：query / notify test / ntfy-info
// ---------------------------------------------------------------------------

fn run_query_cli() -> Result<String, String> {
    let config = config::load_or_create()?;
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(async move {
        let results = quota::query_all(&config).await;
        if results.is_empty() {
            return Ok("没有启用的额度 Provider（编辑 config.toml 填入 API Key）".to_string());
        }
        let notification = quota::render_notification(&results);
        println!("{}", notification.body);
        // query 命令也走一次推送，方便远程验收
        push_notification(&config, &notification).await;
        Ok(notification.body)
    })
}

fn run_notify_cli(arguments: &[String]) -> Result<String, String> {
    match arguments.first().map(String::as_str) {
        Some("test") => {
            let config = config::load_or_create()?;
            let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
            let delivered = runtime.block_on(async {
                push_notification(
                    &config,
                    &ntfy::Notification {
                        title: "MiBandNotifier 测试通知".to_string(),
                        body: "链路正常：PC toast + 手环应同步震动".to_string(),
                        tags: vec!["white_check_mark".to_string()],
                        priority: 4,
                    },
                )
                .await
            });
            match delivered {
                PushOutcome::Both => Ok("测试通知已发送（toast + ntfy）".to_string()),
                PushOutcome::ToastOnly => Err("toast 已弹，但 ntfy 推送失败（查服务/防火墙）".to_string()),
                PushOutcome::NtfyOnly => Ok("ntfy 已推送（toast 被禁用或失败）".to_string()),
                PushOutcome::None => Err("toast 与 ntfy 均失败".to_string()),
            }
        }
        Some(other) => Err(format!("未知 notify 子命令: {other}（仅支持 notify test）")),
        None => Err("用法: MiBandNotifier notify test".to_string()),
    }
}

fn run_ntfy_info() -> Result<String, String> {
    let config = config::load_or_create()?;
    let topic = if config.ntfy.topic.is_empty() { "<未生成>" } else { &config.ntfy.topic };
    let mut output = format!(
        "ntfy 服务器: {}\n订阅主题: {}\n\n手机 ntfy App 订阅地址（任选其一）:",
        config.ntfy.server_url, topic
    );
    let lan_addresses: Vec<String> = local_addresses()
        .into_iter()
        .filter(|interface| interface.ip().is_ipv4())
        .map(|interface| interface.ip().to_string())
        .collect();
    if lan_addresses.is_empty() {
        output.push_str("\n  <未检测到局域网 IPv4 地址>");
    } else {
        for address in lan_addresses {
            output.push_str(&format!("\n  http://{address}:{}/{topic}", config::NTFY_PORT));
        }
    }
    output.push_str(
        "\n\n提示:\n\
         \x20 1. 防火墙需放行 TCP 8090 入站（专用网络）\n\
         \x20 2. 手机与电脑须同一局域网\n\
         \x20 3. 小米运动健康 → APP通知提醒 → 勾选 ntfy 应用"
    );
    println!("{output}");
    Ok(output)
}

fn local_addresses() -> Vec<if_addrs::Interface> {
    if_addrs::get_if_addrs().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// 推送统一出口（toast + ntfy）
// ---------------------------------------------------------------------------

pub enum PushOutcome {
    Both,
    ToastOnly,
    NtfyOnly,
    None,
}

async fn push_notification(config: &config::Config, notification: &ntfy::Notification) -> PushOutcome {
    let toast_ok = if config.toast {
        toast::show(&toast::ToastContent {
            title: notification.title.clone(),
            body: notification.body.clone(),
        })
        .is_ok()
    } else {
        false
    };
    let ntfy_ok = match ntfy::NtfyClient::new(&config.ntfy) {
        Ok(client) => client.publish(notification).await.unwrap_or(false),
        Err(_) => false,
    };
    match (toast_ok, ntfy_ok) {
        (true, true) => PushOutcome::Both,
        (true, false) => PushOutcome::ToastOnly,
        (false, true) => PushOutcome::NtfyOnly,
        (false, false) => PushOutcome::None,
    }
}

// ---------------------------------------------------------------------------
// 常驻托盘模式
// ---------------------------------------------------------------------------

/// 托盘菜单命令（WM_COMMAND）
#[cfg(windows)]
const MENU_QUERY_NOW: u32 = 1001;
#[cfg(windows)]
const MENU_TOGGLE_PAUSE: u32 = 1002;
#[cfg(windows)]
const MENU_QUIT: u32 = 1003;

fn run_tray() -> Result<(), String> {
    let config = config::load_or_create()?;
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;

    // 推送通道：调度器/托盘菜单 → 消费者统一推送（toast + ntfy）
    let (notification_tx, notification_rx) = tokio::sync::mpsc::channel::<ntfy::Notification>(64);

    let service = Arc::new(AppService {
        config: config.clone(),
        pause_push: std::sync::atomic::AtomicBool::new(false),
        quota_summary: Arc::new(Mutex::new(String::from("额度未查询"))),
        notification_tx,
        runtime,
    });
    APP.set(service.clone()).map_err(|_| "重复初始化".to_string())?;

    // 后台任务：spool 轮询 + 额度调度 + 推送消费者
    start_background(service.clone(), notification_rx);

    #[cfg(windows)]
    unsafe {
        register_tray_window_class()?;
        let window = create_tray_window()?;
        let _icon = add_tray_icon(window)?;
        let mut message = MSG::default();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        remove_tray_icon(window);
    }
    #[cfg(not(windows))]
    {
        // 非 Windows 平台退化：阻塞等待（便于跨平台单元测试编译）
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
    Ok(())
}

fn start_background(
    service: Arc<AppService>,
    mut notification_rx: tokio::sync::mpsc::Receiver<ntfy::Notification>,
) {
    let spool_dir = config::data_dir().join("hook-spool");
    // 启动时清理 7 天前的事件
    hook::prune_stale_events(&spool_dir, chrono::Duration::days(7));
    let spool_service = service.clone();
    service.runtime.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(1500));
        loop {
            interval.tick().await;
            for record in hook::drain_events(&spool_dir) {
                handle_hook_record(&spool_service, &record).await;
            }
        }
    });
    // 推送消费者：统一走 toast + ntfy 出口
    let push_config = service.config.clone();
    service.runtime.spawn(async move {
        while let Some(notification) = notification_rx.recv().await {
            let _ = push_notification(&push_config, &notification).await;
        }
    });
    // 额度调度（告警经通道推送）
    let scheduler_config = service.config.clone();
    let scheduler_tx = service.notification_tx.clone();
    service.runtime.spawn(async move {
        quota::run_scheduler(scheduler_config, scheduler_tx).await;
    });
}

async fn handle_hook_record(service: &Arc<AppService>, record: &hook::SpoolRecord) {
    let enabled = match record.event {
        hook::HookEvent::Stop => service.config.events.on_stop,
        hook::HookEvent::PermissionRequest => service.config.events.on_permission,
    };
    if !enabled {
        return;
    }
    if service
        .pause_push
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let summary = record.summary_line();
    let body = if summary.is_empty() {
        record.event.display_title().to_string()
    } else {
        summary
    };
    let notification = ntfy::Notification {
        title: format!("zcode · {}", record.event.display_title()),
        body,
        tags: match record.event {
            hook::HookEvent::Stop => vec!["tada".to_string()],
            hook::HookEvent::PermissionRequest => vec!["warning".to_string()],
        },
        priority: match record.event {
            hook::HookEvent::Stop => 4,
            hook::HookEvent::PermissionRequest => 5,
        },
    };
    let _ = service.notification_tx.send(notification).await;
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::repeat(0).take(1)).collect()
}

#[cfg(windows)]
unsafe fn register_tray_window_class() -> Result<(), String> {
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let class_name = wide("MiBandNotifierTrayWindow");
    let tray_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(tray_window_proc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
        lpszClassName: class_name.as_ptr(),
        ..Default::default()
    };
    if unsafe { RegisterClassExW(&tray_class) } == 0 {
        return Err("无法注册托盘窗口类".to_string());
    }
    Ok(())
}

#[cfg(windows)]
unsafe fn create_tray_window() -> Result<HWND, String> {
    let class_name = wide("MiBandNotifierTrayWindow");
    let title = wide(APP_NAME);
    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        )
    };
    if window.is_null() {
        Err("无法创建托盘窗口".to_string())
    } else {
        Ok(window)
    }
}

#[cfg(windows)]
unsafe fn tray_icon_data(window: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: window,
        uID: TRAY_ICON_ID,
        ..Default::default()
    }
}

#[cfg(windows)]
unsafe fn add_tray_icon(window: HWND) -> Result<(), String> {
    let mut data = unsafe { tray_icon_data(window) };
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_MESSAGE;
    // 使用应用默认图标（LoadIconW 已在 windows-sys 0.61 移除便捷常量，取 IDI_APPLICATION）
    let icon = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::LoadIconW(
            std::ptr::null_mut(),
            windows_sys::Win32::UI::WindowsAndMessaging::IDI_APPLICATION,
        )
    };
    data.hIcon = icon;
    copy_wide(&mut data.szTip, "MiBandNotifier");
    if unsafe { Shell_NotifyIconW(NIM_ADD, &data) } == 0 {
        return Err("无法创建通知区域图标".to_string());
    }
    Ok(())
}

#[cfg(windows)]
fn copy_wide(buffer: &mut [u16; 128], text: &str) {
    for (slot, unit) in buffer.iter_mut().zip(text.encode_utf16()) {
        *slot = unit;
    }
}

#[cfg(windows)]
unsafe fn remove_tray_icon(window: HWND) {
    let data = unsafe { tray_icon_data(window) };
    unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
}

#[cfg(windows)]
unsafe extern "system" fn tray_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        TRAY_MESSAGE => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP || event == WM_CONTEXTMENU || event == WM_LBUTTONDBLCLK {
                unsafe { show_tray_menu(window) };
            }
            0
        }
        WM_COMMAND => {
            let command = (wparam & 0xFFFF) as u32;
            handle_menu_command(command);
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

#[cfg(windows)]
fn handle_menu_command(command: u32) {
    let Some(service) = APP.get() else { return };
    match command {
        MENU_QUERY_NOW => {
            let config = service.config.clone();
            let sender = service.notification_tx.clone();
            let summary_slot = service.quota_summary.clone();
            service.runtime.spawn(async move {
                let results = quota::query_all(&config).await;
                let notification = quota::render_notification(&results);
                if let Ok(mut summary) = summary_slot.lock() {
                    *summary = notification.body.clone();
                }
                let _ = sender.send(notification).await;
            });
        }
        MENU_TOGGLE_PAUSE => {
            let paused = service
                .pause_push
                .fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
            // 取反后即为当前状态
            let now_paused = !paused;
            let body = if now_paused {
                        "已暂停推送（再点一次恢复）".to_string()
                    } else {
                        "已恢复推送".to_string()
                    };
                let _ = toast::show(&toast::ToastContent {
                    title: APP_NAME.to_string(),
                    body,
                });
        }
        MENU_QUIT => {
            unsafe { PostQuitMessage(0) };
        }
        _ => {}
    }
}

#[cfg(windows)]
unsafe fn show_tray_menu(window: HWND) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, SetForegroundWindow,
        TrackPopupMenu, MF_BYCOMMAND, MF_CHECKED, MF_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
        TPM_RIGHTBUTTON,
    };
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }
    let paused = APP
        .get()
        .map(|service| service.pause_push.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(false);
    let query_label = wide("立即查询额度");
    let pause_label = wide(if paused { "恢复推送" } else { "暂停推送" });
    let quit_label = wide("退出");
    unsafe {
        AppendMenuW(menu, MF_STRING | MF_BYCOMMAND, MENU_QUERY_NOW as usize, query_label.as_ptr());
        AppendMenuW(
            menu,
            MF_STRING | MF_BYCOMMAND | if paused { MF_CHECKED } else { 0 },
            MENU_TOGGLE_PAUSE as usize,
            pause_label.as_ptr(),
        );
        AppendMenuW(menu, MF_STRING | MF_BYCOMMAND, MENU_QUIT as usize, quit_label.as_ptr());
        let mut cursor = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor) != 0 {
            SetForegroundWindow(window);
            TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
                cursor.x,
                cursor.y,
                0,
                window,
                std::ptr::null(),
            );
        }
        DestroyMenu(menu);
    }
}
