//! Coleta local reutilizável: sem janela, eventos Tauri ou threads em segundo plano.

mod cpu;
mod disk;
mod gpu;
mod memory;

use std::{error::Error, time::Instant};

pub use disk::{Volume, VolumeUsage};
pub use gpu::GpuDevice;
use serde::Serialize;
use sysinfo::System;

/// MB e GB são decimais: 1 MB = 1.000.000 bytes e 1 GB = 1.000.000.000 bytes.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub cpu_percent: f32,
    pub ram_used_mb: f64,
    pub ram_total_mb: f64,
    pub ram_percent: f32,
    pub disk: Option<DiskInfo>,
    pub system_volume: Option<String>,
    pub volumes: Vec<VolumeUsage>,
    /// None indica que não há adaptador físico disponível. Sensores são independentes.
    pub gpu: Option<GpuInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub used_percent: f32,
    pub used_gb: f64,
}

/// Cada sensor é independente: placas antigas podem oferecer apenas parte deles.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GpuInfo {
    pub usage_percent: Option<u32>,
    pub vram_used_mb: Option<f64>,
    pub vram_total_mb: Option<f64>,
    pub temperature_c: Option<u32>,
}

#[derive(Clone, Serialize)]
pub struct Inventory {
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub windows_build: Option<String>,
    pub architecture: String,
    pub cpu: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    pub ram_mb: f64,
    pub system_volume: Option<String>,
    pub volumes: Vec<Volume>,
    pub gpus: Vec<GpuDevice>,
}

pub(super) const BYTES_PER_MB: f64 = 1_000_000.0;
pub(super) const BYTES_PER_GB: f64 = 1_000_000_000.0;

/// Mantém o histórico de CPU e a NVML entre amostras, evitando reinicializações caras.
pub struct Collector {
    system: System,
    last_cpu_sample: Instant,
    disk: disk::DiskCollector,
    gpu: gpu::GpuCollector,
}

impl Collector {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut system = System::new()?;
        let last_cpu_sample = cpu::prime(&mut system);
        Ok(Self {
            system,
            last_cpu_sample,
            disk: disk::DiskCollector::new().unwrap_or_else(|error| {
                eprintln!("[volumes] {error}");
                disk::DiskCollector::empty()
            }),
            gpu: gpu::GpuCollector::new(),
        })
    }

    /// A primeira chamada espera o mínimo necessário para ter duas amostras de CPU.
    /// O chamador define a cadência normal (por exemplo, 2 s); aqui não há loop.
    pub fn collect(&mut self) -> Result<Snapshot, Box<dyn Error>> {
        self.collect_metrics(true, &[])
    }

    /// Mantém o mesmo formato do Snapshot, mas não consulta sensores ocultos na UI.
    pub fn collect_for_ui(&mut self, selected: &[String]) -> Result<Snapshot, Box<dyn Error>> {
        self.collect_metrics(false, selected)
    }

    fn collect_metrics(
        &mut self,
        details: bool,
        selected: &[String],
    ) -> Result<Snapshot, Box<dyn Error>> {
        let cpu_percent = cpu::collect(&mut self.system, &mut self.last_cpu_sample);
        let (ram_used_mb, ram_total_mb, ram_percent) = memory::collect(&mut self.system);
        let (disk, volumes) = self.disk.collect(selected);
        Ok(Snapshot {
            cpu_percent,
            ram_used_mb,
            ram_total_mb,
            ram_percent,
            disk,
            system_volume: self.disk.system_letter.clone(),
            volumes,
            gpu: self.gpu.collect(details),
        })
    }

    pub fn inventory(&mut self) -> Inventory {
        let (_, ram_mb, _) = memory::collect(&mut self.system);
        Inventory {
            os: System::long_os_version().ok(),
            os_version: System::os_version().ok(),
            windows_build: System::kernel_version().ok(),
            architecture: System::cpu_arch(),
            cpu: self
                .system
                .cpus()
                .first()
                .map(|cpu| cpu.brand().trim().to_owned())
                .filter(|brand| !brand.is_empty()),
            physical_cores: System::physical_core_count().ok(),
            logical_cores: self.logical_cpu_count(),
            ram_mb,
            system_volume: self.disk.system_letter.clone(),
            volumes: self.disk.inventory(),
            gpus: self.gpu.inventory(),
        }
    }

    /// Após uma pausa, não misturamos todo o período oculto com a leitura atual.
    pub fn reset_cpu_baseline(&mut self) {
        self.last_cpu_sample = cpu::prime(&mut self.system);
    }

    /// Permite comparar a CPU do processo com a capacidade total da máquina.
    pub fn logical_cpu_count(&self) -> usize {
        self.system.cpus().len().max(1)
    }
}
