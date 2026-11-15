//! Coleta local reutilizável: sem janela, eventos Tauri ou threads em segundo plano.

mod cpu;
mod disk;
mod gpu;
mod memory;

use std::{error::Error, time::Instant};

use serde::Serialize;
use sysinfo::System;

/// MB e GB são decimais: 1 MB = 1.000.000 bytes e 1 GB = 1.000.000.000 bytes.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub cpu_percent: f32,
    pub ram_used_mb: f64,
    pub ram_total_mb: f64,
    pub ram_percent: f32,
    pub disk: DiskInfo,
    /// None significa que NVML ou a placa não estão disponíveis; nunca significa 0%.
    pub gpu: Option<GpuInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub used_percent: f32,
    pub used_gb: f64,
}

/// Cada sensor é independente: placas antigas podem oferecer apenas parte deles.
#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub usage_percent: Option<u32>,
    pub vram_used_mb: Option<f64>,
    pub vram_total_mb: Option<f64>,
    pub temperature_c: Option<u32>,
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
            disk: disk::DiskCollector::new()?,
            gpu: gpu::GpuCollector::new(),
        })
    }

    /// A primeira chamada espera o mínimo necessário para ter duas amostras de CPU.
    /// O chamador define a cadência normal (por exemplo, 2 s); aqui não há loop.
    pub fn collect(&mut self) -> Result<Snapshot, Box<dyn Error>> {
        let cpu_percent = cpu::collect(&mut self.system, &mut self.last_cpu_sample);
        let (ram_used_mb, ram_total_mb, ram_percent) = memory::collect(&mut self.system);
        Ok(Snapshot {
            cpu_percent,
            ram_used_mb,
            ram_total_mb,
            ram_percent,
            disk: self.disk.collect()?,
            gpu: self.gpu.collect(),
        })
    }

    /// Permite comparar a CPU do processo com a capacidade total da máquina.
    pub fn logical_cpu_count(&self) -> usize {
        self.system.cpus().len().max(1)
    }
}
