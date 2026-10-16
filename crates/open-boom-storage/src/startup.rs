//! Windows Current User startup run key (`HKCU\...\Run`) management.
//!
//! Provides un-elevated startup launch registration for OpenBoom without requiring
//! administrative elevation (UAC). Includes mock mode for headless testing and
//! non-Windows platforms.

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

/// Standard Windows registry run key path under `HKEY_CURRENT_USER`.
pub const RUN_KEY_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Default application registration identifier.
pub const DEFAULT_APP_NAME: &str = "OpenBoom";

/// Manager for Windows startup persistence via `HKCU\...\Run`.
#[derive(Debug, Clone)]
pub struct StartupManager {
    app_name: String,
    mock: bool,
    mock_store: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for StartupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupManager {
    /// Creates a new `StartupManager` operating on real platform persistence (Windows registry or macOS LaunchAgents).
    pub fn new() -> Self {
        Self {
            app_name: DEFAULT_APP_NAME.to_string(),
            mock: !cfg!(any(windows, target_os = "macos")),
            mock_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a mock `StartupManager` that stores run keys in-memory for testing.
    pub fn mock() -> Self {
        Self {
            app_name: DEFAULT_APP_NAME.to_string(),
            mock: true,
            mock_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a `StartupManager` with a customized application identifier.
    pub fn with_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    /// Application key name used in the registry.
    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    /// Returns whether this manager is running in mock mode.
    pub fn is_mock(&self) -> bool {
        self.mock
    }

    /// Checks if launch on Windows startup is currently registered.
    pub fn is_enabled(&self) -> io::Result<bool> {
        if self.mock {
            let store = self.mock_store.lock().unwrap();
            return Ok(store.contains_key(&self.app_name));
        }

        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
            use windows::Win32::System::Registry::{
                RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
                KEY_QUERY_VALUE,
            };

            let subkey_wide = to_wide_null(RUN_KEY_SUBKEY);
            let name_wide = to_wide_null(&self.app_name);

            unsafe {
                let mut hkey = HKEY::default();
                let status = RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey_wide.as_ptr()),
                    0,
                    KEY_QUERY_VALUE,
                    &mut hkey,
                );

                if status == ERROR_FILE_NOT_FOUND {
                    return Ok(false);
                }
                if status != ERROR_SUCCESS {
                    return Err(io::Error::from_raw_os_error(status.0 as i32));
                }

                let query_status = RegQueryValueExW(
                    hkey,
                    PCWSTR(name_wide.as_ptr()),
                    None,
                    None,
                    None,
                    None,
                );

                let _ = RegCloseKey(hkey);

                if query_status == ERROR_SUCCESS {
                    Ok(true)
                } else if query_status == ERROR_FILE_NOT_FOUND {
                    Ok(false)
                } else {
                    Err(io::Error::from_raw_os_error(query_status.0 as i32))
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            Ok(self.macos_launch_agent_plist_path().exists())
        }

        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            Ok(false)
        }
    }

    /// Registers the given executable path in `HKCU\...\Run` on Windows or `~/Library/LaunchAgents/` on macOS.
    pub fn enable(&self, app_path: &str) -> io::Result<()> {
        if self.mock {
            let mut store = self.mock_store.lock().unwrap();
            store.insert(self.app_name.clone(), app_path.to_string());
            return Ok(());
        }

        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::ERROR_SUCCESS;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
                KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
            };

            let subkey_wide = to_wide_null(RUN_KEY_SUBKEY);
            let name_wide = to_wide_null(&self.app_name);
            let value_wide = to_wide_null(app_path);

            unsafe {
                let mut hkey = HKEY::default();
                let status = RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey_wide.as_ptr()),
                    0,
                    None,
                    REG_OPTION_NON_VOLATILE,
                    KEY_SET_VALUE,
                    None,
                    &mut hkey,
                    None,
                );

                if status != ERROR_SUCCESS {
                    return Err(io::Error::from_raw_os_error(status.0 as i32));
                }

                let byte_len = value_wide.len() * std::mem::size_of::<u16>();
                let byte_slice = std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, byte_len);

                let set_status = RegSetValueExW(
                    hkey,
                    PCWSTR(name_wide.as_ptr()),
                    0,
                    REG_SZ,
                    Some(byte_slice),
                );

                let _ = RegCloseKey(hkey);

                if set_status == ERROR_SUCCESS {
                    Ok(())
                } else {
                    Err(io::Error::from_raw_os_error(set_status.0 as i32))
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let plist_path = self.macos_launch_agent_plist_path();
            if let Some(parent) = plist_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.openboom.launcher</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
"#,
                app_path
            );
            let tmp_path = plist_path.with_extension(format!("tmp.{}", std::process::id()));
            std::fs::write(&tmp_path, plist_content)?;
            std::fs::rename(&tmp_path, &plist_path)?;

            let _ = std::process::Command::new("launchctl")
                .args(["load", plist_path.to_str().unwrap_or("")])
                .output();

            Ok(())
        }

        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            Ok(())
        }
    }

    /// Removes the application startup entry.
    /// Gracefully succeeds if the entry or file is already absent.
    pub fn disable(&self) -> io::Result<()> {
        if self.mock {
            let mut store = self.mock_store.lock().unwrap();
            store.remove(&self.app_name);
            return Ok(());
        }

        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
            use windows::Win32::System::Registry::{
                RegCloseKey, RegDeleteValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
                KEY_SET_VALUE,
            };

            let subkey_wide = to_wide_null(RUN_KEY_SUBKEY);
            let name_wide = to_wide_null(&self.app_name);

            unsafe {
                let mut hkey = HKEY::default();
                let status = RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey_wide.as_ptr()),
                    0,
                    KEY_SET_VALUE,
                    &mut hkey,
                );

                if status == ERROR_FILE_NOT_FOUND {
                    return Ok(());
                }
                if status != ERROR_SUCCESS {
                    return Err(io::Error::from_raw_os_error(status.0 as i32));
                }

                let del_status = RegDeleteValueW(hkey, PCWSTR(name_wide.as_ptr()));
                let _ = RegCloseKey(hkey);

                if del_status == ERROR_SUCCESS || del_status == ERROR_FILE_NOT_FOUND {
                    Ok(())
                } else {
                    Err(io::Error::from_raw_os_error(del_status.0 as i32))
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let plist_path = self.macos_launch_agent_plist_path();
            let _ = std::process::Command::new("launchctl")
                .args(["unload", plist_path.to_str().unwrap_or("")])
                .output();
            if plist_path.exists() {
                let _ = std::fs::remove_file(&plist_path);
            }
            Ok(())
        }

        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            Ok(())
        }
    }

    /// Returns the absolute path to the macOS LaunchAgent plist file (`~/Library/LaunchAgents/com.openboom.launcher.plist`).
    #[cfg(target_os = "macos")]
    pub fn macos_launch_agent_plist_path(&self) -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        std::path::PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join("com.openboom.launcher.plist")
    }

    /// Enables startup launch using the current process executable path.
    pub fn enable_current_exe(&self) -> io::Result<()> {
        let exe_path = std::env::current_exe()?;
        let path_str = exe_path.to_string_lossy();
        let formatted = if path_str.contains(' ') && !path_str.starts_with('"') {
            format!("\"{}\"", path_str)
        } else {
            path_str.to_string()
        };
        self.enable(&formatted)
    }

    /// Returns the currently registered executable path, if any.
    pub fn get_registered_path(&self) -> io::Result<Option<String>> {
        if self.mock {
            let store = self.mock_store.lock().unwrap();
            return Ok(store.get(&self.app_name).cloned());
        }

        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS};
            use windows::Win32::System::Registry::{
                RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
                KEY_QUERY_VALUE, REG_SZ,
            };

            let subkey_wide = to_wide_null(RUN_KEY_SUBKEY);
            let name_wide = to_wide_null(&self.app_name);

            unsafe {
                let mut hkey = HKEY::default();
                let status = RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey_wide.as_ptr()),
                    0,
                    KEY_QUERY_VALUE,
                    &mut hkey,
                );

                if status == ERROR_FILE_NOT_FOUND {
                    return Ok(None);
                }
                if status != ERROR_SUCCESS {
                    return Err(io::Error::from_raw_os_error(status.0 as i32));
                }

                let mut data_len = 512u32;
                let mut buffer: Vec<u8> = vec![0; data_len as usize];
                let mut val_type = REG_SZ;

                let mut query_status = RegQueryValueExW(
                    hkey,
                    PCWSTR(name_wide.as_ptr()),
                    None,
                    Some(&mut val_type),
                    Some(buffer.as_mut_ptr()),
                    Some(&mut data_len),
                );

                if query_status == ERROR_MORE_DATA {
                    buffer.resize(data_len as usize, 0);
                    query_status = RegQueryValueExW(
                        hkey,
                        PCWSTR(name_wide.as_ptr()),
                        None,
                        Some(&mut val_type),
                        Some(buffer.as_mut_ptr()),
                        Some(&mut data_len),
                    );
                }

                let _ = RegCloseKey(hkey);

                if query_status == ERROR_SUCCESS {
                    let u16_slice: &[u16] = std::slice::from_raw_parts(
                        buffer.as_ptr() as *const u16,
                        data_len as usize / 2,
                    );
                    let end = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
                    let string = String::from_utf16_lossy(&u16_slice[..end]);
                    Ok(Some(string))
                } else if query_status == ERROR_FILE_NOT_FOUND {
                    Ok(None)
                } else {
                    Err(io::Error::from_raw_os_error(query_status.0 as i32))
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let plist_path = self.macos_launch_agent_plist_path();
            if !plist_path.exists() {
                return Ok(None);
            }
            let content = std::fs::read_to_string(&plist_path)?;
            if let Some(pos) = content.find("<key>ProgramArguments</key>") {
                let rest = &content[pos..];
                if let Some(s_start) = rest.find("<string>") {
                    let s_start = s_start + "<string>".len();
                    if let Some(s_end) = rest[s_start..].find("</string>") {
                        let path = &rest[s_start..s_start + s_end];
                        return Ok(Some(path.to_string()));
                    }
                }
            }
            Ok(Some(String::new()))
        }

        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            Ok(None)
        }
    }
}

#[cfg(windows)]
fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
