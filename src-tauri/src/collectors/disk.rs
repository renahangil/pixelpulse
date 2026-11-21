use std::{error::Error, io};

use serde::Serialize;
use sysinfo::{Disk, DiskKind, DiskRefreshKind, Disks};

use super::{BYTES_PER_GB, DiskInfo};

#[derive(Clone, Serialize)]
pub struct Volume {
    pub id: String,
    pub letter: String,
    pub label: String,
    pub media: &'static str,
    pub system: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct VolumeUsage {
    pub id: String,
    pub letter: String,
    pub usage: Option<DiskInfo>,
}

pub(super) struct DiskCollector {
    disks: Vec<(Volume, Disk)>,
    pub system_letter: Option<String>,
}

impl DiskCollector {
    pub(super) fn new() -> Result<Self, Box<dyn Error>> {
        let system_letter = system_letter();
        let disks = Vec::<Disk>::from(Disks::new_with_refreshed_list_specifics(
            DiskRefreshKind::nothing().with_kind(),
        )?)
        .into_iter()
        .filter_map(|disk| {
            let letter = disk.mount_point().to_str()?.to_uppercase();
            if !is_drive_letter(&letter) {
                return None;
            }
            let system = Some(&letter) == system_letter.as_ref();
            let volume = Volume {
                id: volume_id(&letter).unwrap_or_else(|| letter.clone()),
                letter,
                label: disk.name().to_string_lossy().into_owned(),
                media: match disk.kind() {
                    DiskKind::SSD => "ssd",
                    DiskKind::HDD => "hdd",
                    _ => "unknown",
                },
                system,
            };
            Some((volume, disk))
        })
        .collect();
        let mut result = Self {
            disks,
            system_letter,
        };
        result
            .disks
            .sort_by(|(a, _), (b, _)| a.letter.cmp(&b.letter));
        Ok(result)
    }

    pub fn empty() -> Self {
        Self {
            disks: Vec::new(),
            system_letter: system_letter(),
        }
    }

    pub fn inventory(&self) -> Vec<Volume> {
        self.disks
            .iter()
            .map(|(volume, _)| volume.clone())
            .collect()
    }

    pub(super) fn collect(&mut self, selected: &[String]) -> (Option<DiskInfo>, Vec<VolumeUsage>) {
        let mut system = None;
        let mut extras = Vec::new();
        for (volume, disk) in &mut self.disks {
            if !volume.system && !selected.contains(&volume.id) {
                continue;
            }
            // A volume can disappear or fail without invalidating CPU, RAM or other volume readings.
            let usage = disk
                .refresh_specifics(DiskRefreshKind::nothing().with_storage())
                .then(|| from_space(disk.total_space(), disk.available_space()).ok())
                .flatten();
            if volume.system {
                system = usage;
            } else {
                extras.push(VolumeUsage {
                    id: volume.id.clone(),
                    letter: volume.letter.clone(),
                    usage,
                });
            }
        }
        (system, extras)
    }
}

fn is_drive_letter(letter: &str) -> bool {
    matches!(letter.as_bytes(), [drive, b':', b'\\'] if drive.is_ascii_uppercase())
}

fn system_letter() -> Option<String> {
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::GetWindowsDirectoryW;
        let mut buffer = [0_u16; 260];
        // SAFETY: The bounded buffer is valid. The drive root takes three characters.
        let len = unsafe { GetWindowsDirectoryW(Some(&mut buffer)) } as usize;
        if (3..buffer.len()).contains(&len) {
            let drive = String::from_utf16_lossy(&buffer[..3]).to_uppercase();
            if is_drive_letter(&drive) {
                return Some(drive);
            }
        }
        let drive = format!("{}\\", std::env::var("SystemDrive").ok()?.to_uppercase());
        is_drive_letter(&drive).then_some(drive)
    }
    #[cfg(not(windows))]
    None
}

fn volume_id(letter: &str) -> Option<String> {
    #[cfg(windows)]
    {
        use windows::{
            Win32::Storage::FileSystem::GetVolumeNameForVolumeMountPointW, core::PCWSTR,
        };
        let root: Vec<_> = letter.encode_utf16().chain([0]).collect();
        let mut name = [0_u16; 64];
        // SAFETY: Input is NUL-terminated and the output buffer is valid.
        unsafe { GetVolumeNameForVolumeMountPointW(PCWSTR(root.as_ptr()), &mut name).ok()? };
        Some(String::from_utf16_lossy(
            &name[..name.iter().position(|&unit| unit == 0)?],
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = letter;
        None
    }
}

fn from_space(total: u64, available: u64) -> io::Result<DiskInfo> {
    if total == 0 || available > total {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid volume capacity",
        ));
    }
    let used = total - available;
    Ok(DiskInfo {
        used_percent: (used as f64 / total as f64 * 100.0) as f32,
        used_gb: used as f64 / BYTES_PER_GB,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disco_calcula_espaco_usado_e_rejeita_capacidade_invalida() {
        let info = from_space(1_000_000_000_000, 250_000_000_000).unwrap();
        assert_eq!(info.used_gb, 750.0);
        assert_eq!(info.used_percent, 75.0);
        assert!(from_space(0, 0).is_err());
        assert!(from_space(100, 101).is_err());
        assert!(is_drive_letter("D:\\"));
        assert!(!is_drive_letter("\\\\server\\share"));
    }
}
