use std::{error::Error, io, path::PathBuf};

use sysinfo::{Disk, DiskRefreshKind, Disks};

use super::{BYTES_PER_GB, DiskInfo};

pub(super) struct DiskCollector {
    disk: Disk,
}

impl DiskCollector {
    pub(super) fn new() -> Result<Self, Box<dyn Error>> {
        let root = system_root()?;
        // Enumeramos volumes uma vez; nas amostras só consultamos espaço do selecionado.
        let disks = Vec::<Disk>::from(Disks::new_with_refreshed_list_specifics(
            DiskRefreshKind::nothing(),
        )?);
        let disk = disks
            .into_iter()
            .find(|disk| {
                if cfg!(windows) {
                    disk.mount_point()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&root.to_string_lossy())
                } else {
                    disk.mount_point() == root
                }
            })
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "volume do sistema não encontrado")
            })?;
        Ok(Self { disk })
    }

    pub(super) fn collect(&mut self) -> io::Result<DiskInfo> {
        // storage é capacidade/espaço livre; io_usage seria atividade de leitura/escrita.
        if !self
            .disk
            .refresh_specifics(DiskRefreshKind::nothing().with_storage())
        {
            return Err(io::Error::other(
                "não foi possível atualizar o disco do sistema",
            ));
        }
        from_space(self.disk.total_space(), self.disk.available_space())
    }
}

fn system_root() -> io::Result<PathBuf> {
    #[cfg(windows)]
    {
        // Não presumimos a unidade C: o Windows pode estar instalado em outra unidade.
        let drive = std::env::var_os("SystemDrive")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "SystemDrive ausente"))?;
        let mut root = PathBuf::from(drive);
        root.push(std::path::MAIN_SEPARATOR.to_string());
        // push("\\") preserva o prefixo da unidade no Windows (por exemplo, D:\\).
        Ok(root)
    }
    #[cfg(not(windows))]
    {
        Ok(PathBuf::from("/"))
    }
}

fn from_space(total: u64, available: u64) -> io::Result<DiskInfo> {
    if total == 0 || available > total {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "capacidade do disco inválida",
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
    use super::from_space;

    #[test]
    fn disco_calcula_espaco_usado_e_rejeita_capacidade_invalida() {
        let info = from_space(1_000_000_000_000, 250_000_000_000).unwrap();
        assert_eq!(info.used_gb, 750.0);
        assert_eq!(info.used_percent, 75.0);
        assert!(from_space(0, 0).is_err());
        assert!(from_space(100, 101).is_err());
    }
}
