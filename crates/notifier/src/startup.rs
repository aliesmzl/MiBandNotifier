//! 开机自启：当前用户注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`。
//!
//! 只写 HKCU（当前用户），无需管理员权限。值内容为带引号的 exe 路径。

#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ,
};

#[cfg(windows)]
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
#[cfg(windows)]
const VALUE_NAME: &str = "MiBandNotifier";

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::repeat(0).take(1)).collect()
}

#[cfg(windows)]
pub fn install(exe_path: &str) -> Result<String, String> {
    let command = format!("\"{exe_path}\"");
    let sub_key = wide(RUN_KEY);
    let value_name = wide(VALUE_NAME);
    let value_data = wide(&command);
    unsafe {
        let mut key: HKEY = std::ptr::null_mut();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key,
        );
        if result != 0 {
            return Err(format!("打开注册表 Run 键失败（code {result}）"));
        }
        let bytes = (value_data.len() * 2) as u32;
        let set_result = RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_data.as_ptr() as *const u8,
            bytes,
        );
        RegCloseKey(key);
        if set_result != 0 {
            return Err(format!("写入注册表失败（code {set_result}）"));
        }
    }
    Ok(format!("已设置开机自启：{command}"))
}

#[cfg(windows)]
pub fn uninstall() -> Result<String, String> {
    let sub_key = wide(RUN_KEY);
    let value_name = wide(VALUE_NAME);
    unsafe {
        let mut key: HKEY = std::ptr::null_mut();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key,
        );
        if result != 0 {
            return Err(format!("打开注册表 Run 键失败（code {result}）"));
        }
        let delete_result = RegDeleteValueW(key, value_name.as_ptr());
        RegCloseKey(key);
        if delete_result != 0 && delete_result != 2 {
            // 2 = ERROR_FILE_NOT_FOUND（本就未设置，视为成功）
            return Err(format!("删除注册表值失败（code {delete_result}）"));
        }
    }
    Ok("已取消开机自启".to_string())
}

/// 查询当前自启命令（供测试与诊断）
#[cfg(windows)]
#[allow(dead_code)]
pub fn current_value() -> Option<String> {
    let sub_key = wide(RUN_KEY);
    let value_name = wide(VALUE_NAME);
    unsafe {
        let mut key: HKEY = std::ptr::null_mut();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            0,
            KEY_READ,
            &mut key,
        );
        if result != 0 {
            return None;
        }
        let mut buffer = [0u16; 512];
        let mut size = (buffer.len() * 2) as u32;
        let query_result = RegQueryValueExW(
            key,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut u8,
            &mut size,
        );
        RegCloseKey(key);
        if query_result != 0 {
            return None;
        }
        let length = (size as usize / 2).min(buffer.len());
        let text: String = buffer[..length]
            .iter()
            .take_while(|&&unit| unit != 0)
            .map(|&unit| unit as u32)
            .map(char::from_u32)
            .flatten()
            .collect();
        Some(text)
    }
}

#[cfg(not(windows))]
pub fn install(_exe_path: &str) -> Result<String, String> {
    Err("开机自启仅支持 Windows".to_string())
}

#[cfg(not(windows))]
pub fn uninstall() -> Result<String, String> {
    Err("开机自启仅支持 Windows".to_string())
}
