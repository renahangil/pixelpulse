#[cfg(windows)]
mod platform {
    use std::os::windows::ffi::OsStrExt;

    use windows::{
        ApplicationModel::{StartupTask, StartupTaskState},
        Win32::{
            Foundation::{ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS},
            Storage::Packaging::Appx::GetCurrentPackageFullName,
            System::{
                Registry::{
                    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, REG_SZ, REG_VALUE_TYPE, RegCloseKey,
                    RegDeleteKeyValueW, RegOpenKeyExW, RegQueryValueExW, RegSetKeyValueW,
                },
                WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
            },
        },
        core::{HSTRING, w},
    };

    const TASK: &str = "PixelPulseStartup";

    pub fn update(value: Option<bool>) -> Result<bool, String> {
        let mut len = 0;
        // SAFETY: A null buffer queries whether this process has an MSIX identity.
        if unsafe { GetCurrentPackageFullName(&mut len, None) } == ERROR_INSUFFICIENT_BUFFER {
            packaged(value)
        } else {
            portable(value)
        }
    }

    fn packaged(value: Option<bool>) -> Result<bool, String> {
        struct Runtime;
        impl Drop for Runtime {
            fn drop(&mut self) {
                // SAFETY: Balance RoInitialize on the same blocking thread.
                unsafe { RoUninitialize() };
            }
        }

        // SAFETY: This runs on a worker thread and Drop performs the cleanup.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| error.to_string())?;
        let _runtime = Runtime;
        let task = StartupTask::GetAsync(&HSTRING::from(TASK))
            .and_then(|op| op.join())
            .map_err(|error| error.to_string())?;
        let state = task.State().map_err(|error| error.to_string())?;
        let enabled = matches!(
            state,
            StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
        );
        match value {
            Some(true) if !enabled => {
                let new_state = task
                    .RequestEnableAsync()
                    .and_then(|op| op.join())
                    .map_err(|error| error.to_string())?;
                if !matches!(
                    new_state,
                    StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
                ) {
                    return Err(
                        "Windows did not allow startup. Check Windows Settings > Apps > Startup."
                            .into(),
                    );
                }
            }
            Some(false) if enabled => task.Disable().map_err(|error| error.to_string())?,
            _ => {}
        }
        task.State()
            .map(|state| {
                matches!(
                    state,
                    StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
                )
            })
            .map_err(|error| error.to_string())
    }

    fn portable(value: Option<bool>) -> Result<bool, String> {
        let run = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let name = w!("PixelPulse");
        // PixelPulse owns this value. A presence check also lets users disable an old entry
        // after moving the standalone executable.
        // SAFETY: The registry receives NUL-terminated strings and explicitly sized buffers.
        unsafe {
            if let Some(enable) = value {
                let status = if enable {
                    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
                    let command: Vec<u16> = [u16::from(b'"')]
                        .into_iter()
                        .chain(exe.as_os_str().encode_wide())
                        .chain([u16::from(b'"'), 0])
                        .collect();
                    RegSetKeyValueW(
                        HKEY_CURRENT_USER,
                        run,
                        name,
                        REG_SZ.0,
                        Some(command.as_ptr().cast()),
                        (command.len() * 2) as u32,
                    )
                } else {
                    RegDeleteKeyValueW(HKEY_CURRENT_USER, run, name)
                };
                if status != ERROR_SUCCESS && (enable || status != ERROR_FILE_NOT_FOUND) {
                    return Err(status.to_hresult().message());
                }
            }

            let mut key = HKEY::default();
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, run, None, KEY_QUERY_VALUE, &mut key);
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }
            if status != ERROR_SUCCESS {
                return Err(status.to_hresult().message());
            }

            // RegQueryValueExW reads metadata without expanding or comparing the path.
            // Close the handle even if the query fails.
            let mut size = 0;
            let mut kind = REG_VALUE_TYPE::default();
            let status = RegQueryValueExW(key, name, None, Some(&mut kind), None, Some(&mut size));
            let _ = RegCloseKey(key);
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }
            if status != ERROR_SUCCESS {
                return Err(status.to_hresult().message());
            }
            Ok(kind == REG_SZ && size > 2)
        }
    }
}

#[cfg(windows)]
pub use platform::update;

#[cfg(not(windows))]
pub fn update(_: Option<bool>) -> Result<bool, String> {
    Err("Startup is available on Windows.".into())
}
