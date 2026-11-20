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
                    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW,
                    RegSetKeyValueW,
                },
                WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
            },
        },
        core::{HSTRING, w},
    };

    const TASK: &str = "PixelPulseStartup";

    pub fn update(value: Option<bool>) -> Result<bool, String> {
        let mut len = 0;
        // SAFETY: buffer nulo consulta apenas se o processo tem identidade MSIX.
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
                // SAFETY: equilibra RoInitialize na mesma thread de bloqueio.
                unsafe { RoUninitialize() };
            }
        }

        // SAFETY: chamada feita numa thread de trabalho, com finalização via Drop.
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
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let command: Vec<u16> = [u16::from(b'"')]
            .into_iter()
            .chain(exe.as_os_str().encode_wide())
            .chain([u16::from(b'"'), 0])
            .collect();

        // SAFETY: Registry APIs receive valid UTF-16 and sized buffers. Only our
        // per-user PixelPulse value is read or changed, without elevation.
        unsafe {
            if let Some(enable) = value {
                let status = if enable {
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
            let mut size = 0;
            let status = RegGetValueW(
                HKEY_CURRENT_USER,
                run,
                name,
                RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            );
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }
            if status != ERROR_SUCCESS {
                return Err(status.to_hresult().message());
            }
            if size as usize != command.len() * 2 {
                return Ok(false);
            }
            let mut actual = vec![0_u16; command.len()];
            let status = RegGetValueW(
                HKEY_CURRENT_USER,
                run,
                name,
                RRF_RT_REG_SZ,
                None,
                Some(actual.as_mut_ptr().cast()),
                Some(&mut size),
            );
            if status != ERROR_SUCCESS {
                return Err(status.to_hresult().message());
            }
            Ok(actual == command)
        }
    }
}

#[cfg(windows)]
pub use platform::update;

#[cfg(not(windows))]
pub fn update(_: Option<bool>) -> Result<bool, String> {
    Err("Startup is available on Windows.".into())
}
