use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureSensor};
use serde::Serialize;

#[cfg(windows)]
use windows::{
    Win32::{
        Foundation::LUID,
        Graphics::DXCore::{
            DXCORE_ADAPTER_ATTRIBUTE_D3D11_GRAPHICS, DXCORE_ADAPTER_ATTRIBUTE_D3D12_CORE_COMPUTE,
            DXCoreAdapterProperty as Property, DXCoreCreateAdapterFactory, DXCoreHardwareID,
            DriverDescription, HardwareID, IDXCoreAdapter, IDXCoreAdapterFactory,
            IDXCoreAdapterList, InstanceLuid, IsHardware, IsIntegrated,
        },
        System::Performance::*,
    },
    core::{PCWSTR, w},
};

use super::{BYTES_PER_MB, GpuInfo};

#[derive(Clone, Debug, Serialize)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: &'static str,
    pub integrated: Option<bool>,
    pub primary: bool,
    pub physical_count: u32,
}

struct Adapter {
    info: GpuDevice,
    nvml_index: Option<u32>,
    #[cfg(windows)]
    luid: Option<LUID>,
    #[cfg(windows)]
    hardware_id: Option<DXCoreHardwareID>,
}

pub(super) struct GpuCollector {
    nvml: Option<Nvml>,
    adapters: Vec<Adapter>,
    #[cfg(windows)]
    pdh: Option<GpuCounter>,
    #[cfg(windows)]
    pdh_attempted: bool,
}

impl GpuCollector {
    pub(super) fn new() -> Self {
        let nvml = Nvml::init().ok();
        #[cfg(windows)]
        let mut adapters = windows_adapters();
        #[cfg(not(windows))]
        let mut adapters: Vec<Adapter> = Vec::new();
        if let Some(nvml) = &nvml {
            let nvidia: Vec<_> = (0..nvml.device_count().unwrap_or_default())
                .filter_map(|index| {
                    let device = nvml.device_by_index(index).ok()?;
                    Some((index, device.name().ok()?, device.pci_info().ok()))
                })
                .collect();
            #[cfg(windows)]
            for adapter in &mut adapters {
                let Some(hardware) = &adapter.hardware_id else {
                    continue;
                };
                if hardware.vendorID != 0x10de {
                    continue;
                }
                let same_card = |pci: &nvml_wrapper::struct_wrappers::device::PciInfo| {
                    pci.pci_device_id >> 16 == hardware.deviceID
                        && pci
                            .pci_sub_system_id
                            .is_none_or(|id| id == hardware.subSysID)
                };
                // Em placas idênticas, não atribuir índices por coincidência. PDH usa o LUID.
                let peers = nvidia
                    .iter()
                    .filter(|(_, _, pci)| pci.as_ref().is_some_and(same_card));
                let mut candidates = peers.map(|(index, _, _)| *index);
                if let Some(index) = candidates.next()
                    && candidates.next().is_none()
                {
                    adapter.nvml_index = Some(index);
                }
            }
            #[cfg(windows)]
            {
                // Um match ambíguo em ambas as direções usa apenas o contador do Windows.
                let indexes: Vec<_> = adapters.iter().filter_map(|a| a.nvml_index).collect();
                for adapter in &mut adapters {
                    if let Some(index) = adapter.nvml_index
                        && indexes.iter().filter(|&&other| other == index).count() > 1
                    {
                        adapter.nvml_index = None;
                    }
                }
            }
            if !adapters
                .iter()
                .any(|adapter| adapter.info.vendor == "NVIDIA")
            {
                for (index, name, _) in nvidia {
                    adapters.push(Adapter {
                        info: GpuDevice {
                            name,
                            vendor: "NVIDIA",
                            integrated: Some(false),
                            primary: false,
                            physical_count: 1,
                        },
                        nvml_index: Some(index),
                        #[cfg(windows)]
                        luid: None,
                        #[cfg(windows)]
                        hardware_id: None,
                    });
                }
            }
        }

        if adapters
            .iter()
            .any(|adapter| adapter.info.integrated == Some(false))
        {
            adapters.retain(|adapter| adapter.info.integrated != Some(true));
        }
        // Priorizar a dedicada conhecida. Depois, preferir leitura NVML identificada.
        adapters.sort_by_key(|adapter| {
            (
                adapter.info.integrated != Some(false),
                adapter.nvml_index.is_none(),
            )
        });
        if let Some(primary) = adapters.first_mut() {
            primary.info.primary = true;
        }
        Self {
            nvml,
            adapters,
            #[cfg(windows)]
            pdh: None,
            #[cfg(windows)]
            pdh_attempted: false,
        }
    }

    pub(super) fn inventory(&self) -> Vec<GpuDevice> {
        self.adapters
            .iter()
            .map(|adapter| adapter.info.clone())
            .collect()
    }

    pub(super) fn collect(&mut self, details: bool) -> Option<GpuInfo> {
        let adapter = self.adapters.first()?;
        let mut info = GpuInfo::default();
        if let Some(device) = adapter
            .nvml_index
            .and_then(|index| self.nvml.as_ref()?.device_by_index(index).ok())
        {
            info.usage_percent = device.utilization_rates().ok().map(|usage| usage.gpu);
            if details {
                let memory = device.memory_info().ok();
                info.vram_used_mb = memory.as_ref().map(|m| m.used as f64 / BYTES_PER_MB);
                info.vram_total_mb = memory.as_ref().map(|m| m.total as f64 / BYTES_PER_MB);
                info.temperature_c = device.temperature(TemperatureSensor::Gpu).ok();
            }
        }
        #[cfg(windows)]
        if info.usage_percent.is_none()
            && let Some(luid) = adapter.luid
        {
            if !self.pdh_attempted {
                self.pdh_attempted = true;
                self.pdh = GpuCounter::new(luid);
            }
            info.usage_percent = self.pdh.as_mut().and_then(GpuCounter::sample);
        }
        Some(info)
    }
}

// Esta propriedade foi documentada depois dos metadados do SDK usados pelo crate.
#[cfg(windows)]
const PHYSICAL_ADAPTER_COUNT: Property = Property(15);

#[cfg(windows)]
fn windows_adapters() -> Vec<Adapter> {
    // SAFETY: DXCore retorna interfaces COM gerenciadas pelos bindings. Cada consulta
    // abaixo passa o tipo documentado para a propriedade e um buffer de tamanho exato.
    unsafe {
        let Ok(factory) = DXCoreCreateAdapterFactory::<IDXCoreAdapterFactory>() else {
            return Vec::new();
        };
        let mut adapters: Vec<Adapter> = Vec::new();
        for attribute in [
            DXCORE_ADAPTER_ATTRIBUTE_D3D11_GRAPHICS,
            DXCORE_ADAPTER_ATTRIBUTE_D3D12_CORE_COMPUTE,
        ] {
            let Ok(list) = factory.CreateAdapterList::<IDXCoreAdapterList>(&[attribute]) else {
                continue;
            };
            for index in 0..list.GetAdapterCount() {
                let Ok(adapter) = list.GetAdapter::<IDXCoreAdapter>(index) else {
                    continue;
                };
                if property::<bool>(&adapter, IsHardware) != Some(true) {
                    continue;
                }
                let luid = property::<LUID>(&adapter, InstanceLuid);
                if let Some(luid) = luid
                    && adapters.iter().any(|other| other.luid == Some(luid))
                {
                    continue;
                }
                let id = property::<DXCoreHardwareID>(&adapter, HardwareID);
                let vendor = match id.map(|id| id.vendorID) {
                    Some(0x10de) => "NVIDIA",
                    Some(0x1002) | Some(0x1022) => "AMD",
                    Some(0x8086) => "Intel",
                    _ => "Unknown",
                };
                let name = adapter
                    .GetPropertySize(DriverDescription)
                    .ok()
                    .filter(|len| (1..=4096).contains(len))
                    .and_then(|len| {
                        let mut bytes = vec![0_u8; len];
                        adapter
                            .GetProperty(DriverDescription, len, bytes.as_mut_ptr().cast())
                            .ok()?;
                        let end = bytes.iter().position(|&byte| byte == 0).unwrap_or(len);
                        Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
                    })
                    .unwrap_or_else(|| vendor.to_owned());
                let integrated = property::<bool>(&adapter, IsIntegrated);
                let physical_count = property::<u32>(&adapter, PHYSICAL_ADAPTER_COUNT)
                    .filter(|count| (1..=32).contains(count))
                    .unwrap_or(1);
                adapters.push(Adapter {
                    info: GpuDevice {
                        name,
                        vendor,
                        integrated,
                        primary: false,
                        physical_count,
                    },
                    nvml_index: None,
                    luid,
                    hardware_id: id,
                });
            }
        }
        adapters
    }
}

#[cfg(windows)]
fn property<T: Default>(adapter: &IDXCoreAdapter, property: Property) -> Option<T> {
    let mut result = T::default();
    // SAFETY: apenas os chamadores acima usam esta função, com o T documentado para cada propriedade.
    unsafe {
        if !adapter.IsPropertySupported(property) {
            return None;
        }
        adapter
            .GetProperty(
                property,
                size_of::<T>(),
                std::ptr::from_mut(&mut result).cast(),
            )
            .ok()?;
    }
    Some(result)
}

#[cfg(windows)]
struct GpuCounter {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    luid: String,
    buffer: Vec<usize>,
}

#[cfg(windows)]
impl GpuCounter {
    fn new(luid: LUID) -> Option<Self> {
        let mut query = PDH_HQUERY::default();
        let mut counter = PDH_HCOUNTER::default();
        // SAFETY: handles são usados só na thread coletora e o query é liberado no Drop.
        unsafe {
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != 0 {
                return None;
            }
            if PdhAddEnglishCounterW(
                query,
                w!(r"\GPU Engine(*)\Utilization Percentage"),
                0,
                &mut counter,
            ) != 0
                || PdhCollectQueryData(query) != 0
            {
                PdhCloseQuery(query);
                return None;
            }
        }
        Some(Self {
            query,
            counter,
            luid: format!(
                "_luid_0x{:08x}_0x{:08x}_",
                luid.HighPart as u32, luid.LowPart
            ),
            buffer: Vec::new(),
        })
    }

    fn sample(&mut self) -> Option<u32> {
        // SAFETY: PDH fornece o tamanho. Vec<usize> alinha os itens e os nomes
        // apontam para o mesmo buffer, mantido vivo enquanto as instâncias são lidas.
        unsafe {
            if PdhCollectQueryData(self.query) != 0 {
                return None;
            }
            let mut bytes = 0;
            let mut count = 0;
            if PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut bytes,
                &mut count,
                None,
            ) != PDH_MORE_DATA
                || bytes == 0
                || bytes > 4_000_000
            {
                return None;
            }
            self.buffer
                .resize((bytes as usize).div_ceil(size_of::<usize>()), 0);
            let items = self
                .buffer
                .as_mut_ptr()
                .cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
            if PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut bytes,
                &mut count,
                Some(items),
            ) != 0
                || count as usize
                    > self.buffer.len() * size_of::<usize>()
                        / size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>()
            {
                return None;
            }
            let mut engines = std::collections::HashMap::<String, f64>::new();
            for entry in std::slice::from_raw_parts(items, count as usize) {
                if !matches!(entry.FmtValue.CStatus, 0 | 1) || entry.szName.is_null() {
                    continue;
                }
                let Ok(name) = entry.szName.to_string() else {
                    continue;
                };
                let Some((_, engine)) = name.split_once(&self.luid) else {
                    continue;
                };
                let value = entry.FmtValue.Anonymous.doubleValue;
                if value.is_finite() {
                    *engines.entry(engine.to_owned()).or_default() += value.max(0.0);
                }
            }
            // Cada instância é um processo em um motor. Somar por motor e pegar
            // o mais ocupado evita somar, por exemplo, 3D com decodificação de vídeo.
            engines
                .into_values()
                .reduce(f64::max)
                .map(|v| v.clamp(0.0, 100.0).round() as u32)
        }
    }
}

#[cfg(windows)]
impl Drop for GpuCounter {
    fn drop(&mut self) {
        // SAFETY: query pertence exclusivamente a esta instância.
        unsafe { PdhCloseQuery(self.query) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_ausente_nao_causa_falha() {
        let mut gpu = GpuCollector {
            nvml: None,
            adapters: Vec::new(),
            #[cfg(windows)]
            pdh: None,
            #[cfg(windows)]
            pdh_attempted: false,
        };
        assert!(gpu.collect(true).is_none());
        assert!(gpu.collect(false).is_none());
    }
}
