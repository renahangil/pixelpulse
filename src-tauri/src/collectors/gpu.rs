use nvml_wrapper::{
    Nvml, enum_wrappers::device::TemperatureSensor, error::NvmlError,
    struct_wrappers::device::MemoryInfo,
};

use super::{BYTES_PER_MB, GpuInfo};

pub(super) struct GpuCollector {
    nvml: Option<Nvml>,
}

impl GpuCollector {
    pub(super) fn new() -> Self {
        // Carregar a DLL e seus símbolos é caro; tentamos uma vez por coletor.
        Self {
            nvml: Nvml::init().ok(),
        }
    }

    pub(super) fn collect(&self, details: bool) -> Option<GpuInfo> {
        let device = self.nvml.as_ref()?.device_by_index(0).ok()?;
        // A ilha só exibe uso: não consultar VRAM/temperatura economiza chamadas à DLL.
        // O CLI mantém todos os sensores para diagnóstico, sem alterar seu contrato.
        if !details {
            return Some(GpuInfo {
                usage_percent: device.utilization_rates().ok().map(|usage| usage.gpu),
                vram_used_mb: None,
                vram_total_mb: None,
                temperature_c: None,
            });
        }
        Some(sensor_info(
            device.utilization_rates().map(|usage| usage.gpu),
            device.memory_info(),
            device.temperature(TemperatureSensor::Gpu),
        ))
    }
}

fn sensor_info(
    usage: Result<u32, NvmlError>,
    memory: Result<MemoryInfo, NvmlError>,
    temperature: Result<u32, NvmlError>,
) -> GpuInfo {
    // O erro de um sensor não descarta sensores válidos, nem inventa valor zero.
    let memory = memory.ok();
    GpuInfo {
        usage_percent: usage.ok(),
        vram_used_mb: memory.as_ref().map(|info| info.used as f64 / BYTES_PER_MB),
        vram_total_mb: memory.as_ref().map(|info| info.total as f64 / BYTES_PER_MB),
        temperature_c: temperature.ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::{GpuCollector, NvmlError, sensor_info};

    #[test]
    fn gpu_ausente_e_sensores_parciais_nao_causam_falha() {
        assert!(GpuCollector { nvml: None }.collect(true).is_none());
        assert!(GpuCollector { nvml: None }.collect(false).is_none());
        let info = sensor_info(
            Err(NvmlError::NotSupported),
            Err(NvmlError::NotSupported),
            Ok(43),
        );
        let json = serde_json::to_value(info).unwrap();
        assert!(json["usage_percent"].is_null());
        assert!(json["vram_used_mb"].is_null());
        assert!(json["vram_total_mb"].is_null());
        assert_eq!(json["temperature_c"], 43);
        let info = sensor_info(
            Ok(7),
            Err(NvmlError::NotSupported),
            Err(NvmlError::NotSupported),
        );
        assert_eq!(info.usage_percent, Some(7));
        assert_eq!(info.temperature_c, None);
    }
}
