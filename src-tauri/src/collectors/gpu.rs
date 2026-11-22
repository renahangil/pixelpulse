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
    retry_at: Option<std::time::Instant>,
    #[cfg(windows)]
    diagnostics: CounterDiagnostics,
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
                // Identical adapters can have unrelated indices. PDH identifies them by LUID.
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
                // Use the Windows counter when the match is ambiguous in either direction.
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
        // Prefer a known discrete adapter, then an unambiguous NVML match.
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
            retry_at: None,
            #[cfg(windows)]
            diagnostics: CounterDiagnostics::default(),
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
        let mut info = GpuInfo {
            integrated: adapter.info.integrated,
            ..Default::default()
        };
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
            let now = std::time::Instant::now();
            if let Some(counter) = &mut self.pdh {
                info.usage_percent = counter.sample(&mut self.diagnostics);
                if info.usage_percent.is_some() {
                    self.retry_at = None;
                } else {
                    // Give new process instances time to accumulate a second sample.
                    let deadline = self.retry_at.get_or_insert(now + RETRY);
                    if now >= *deadline {
                        self.pdh = None;
                        self.retry_at = None;
                    }
                }
            } else if self.retry_at.is_none_or(|deadline| now >= deadline) {
                self.diagnostics = CounterDiagnostics {
                    luid_high: luid.HighPart as u32,
                    luid_low: luid.LowPart,
                    ..Default::default()
                };
                self.pdh = GpuCounter::new(luid, &mut self.diagnostics);
                self.retry_at = Some(now + RETRY);
            }
            if details {
                info.pdh = Some(self.diagnostics.clone());
            }
        }
        Some(info)
    }
}

// Microsoft documented this property after the SDK metadata used by these bindings.
#[cfg(windows)]
const PHYSICAL_ADAPTER_COUNT: Property = Property(15);

#[cfg(windows)]
fn windows_adapters() -> Vec<Adapter> {
    // SAFETY: The bindings manage the COM interfaces returned by DXCore. Each query uses
    // the property's documented data type and an exactly sized buffer.
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
    // SAFETY: The callers above use the documented T for each property.
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
const RETRY: std::time::Duration = std::time::Duration::from_secs(30);

#[cfg(windows)]
#[derive(Clone, Default, Debug, Serialize)]
pub struct CounterDiagnostics {
    luid_high: u32,
    luid_low: u32,
    open: Option<u32>,
    add: Option<u32>,
    collect: Option<u32>,
    size: Option<u32>,
    format: Option<u32>,
    total: u32,
    valid: u32,
    matching: u32,
}

#[cfg(windows)]
fn matching_engine(name: &str, high: u32, low: u32) -> Option<&str> {
    // PDH instance names contain two hexadecimal DWORDs, independent of locale.
    let mut fields = name.split('_');
    fields.find(|field| field.eq_ignore_ascii_case("luid"))?;
    let hex = |field: &str| {
        u32::from_str_radix(
            field
                .strip_prefix("0x")
                .or_else(|| field.strip_prefix("0X"))?,
            16,
        )
        .ok()
    };
    if hex(fields.next()?)? != high || hex(fields.next()?)? != low {
        return None;
    }
    let physical = fields.next()?;
    if !physical.eq_ignore_ascii_case("phys") {
        return None;
    }
    // Slice at the physical-engine suffix, dropping the per-process prefix.
    Some(&name[(physical.as_ptr() as usize - name.as_ptr() as usize)..])
}

#[cfg(windows)]
struct GpuCounter {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    luid: LUID,
    buffer: Vec<usize>,
}

#[cfg(windows)]
impl GpuCounter {
    fn new(luid: LUID, diag: &mut CounterDiagnostics) -> Option<Self> {
        let mut query = PDH_HQUERY::default();
        let mut counter = PDH_HCOUNTER::default();
        // SAFETY: Only the sampler thread uses these handles. Drop closes the query.
        unsafe {
            let status = PdhOpenQueryW(PCWSTR::null(), 0, &mut query);
            diag.open = Some(status);
            if status != 0 {
                return None;
            }
            let status = PdhAddEnglishCounterW(
                query,
                w!(r"\GPU Engine(*)\Utilization Percentage"),
                0,
                &mut counter,
            );
            diag.add = Some(status);
            if status == 0 {
                diag.collect = Some(PdhCollectQueryData(query));
            }
            if diag.collect != Some(0) {
                PdhCloseQuery(query);
                return None;
            }
        }
        Some(Self {
            query,
            counter,
            luid,
            buffer: Vec::new(),
        })
    }

    fn sample(&mut self, diag: &mut CounterDiagnostics) -> Option<u32> {
        // SAFETY: PDH reports the size. Vec<usize> aligns the items, and their names point into
        // the same buffer, which stays alive while the instances are read.
        unsafe {
            diag.size = None;
            diag.format = None;
            diag.total = 0;
            diag.valid = 0;
            diag.matching = 0;
            diag.collect = Some(PdhCollectQueryData(self.query));
            if diag.collect != Some(0) {
                return None;
            }
            let mut bytes = 0;
            let mut count = 0;
            let status = PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut bytes,
                &mut count,
                None,
            );
            diag.size = Some(status);
            if status != PDH_MORE_DATA || bytes == 0 || bytes > 4_000_000 {
                return None;
            }
            self.buffer
                .resize((bytes as usize).div_ceil(size_of::<usize>()), 0);
            let items = self
                .buffer
                .as_mut_ptr()
                .cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
            let status = PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut bytes,
                &mut count,
                Some(items),
            );
            diag.format = Some(status);
            if status != 0
                || count as usize
                    > self.buffer.len() * size_of::<usize>()
                        / size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>()
            {
                return None;
            }
            diag.total = count;
            let mut engines = std::collections::HashMap::<String, f64>::new();
            for entry in std::slice::from_raw_parts(items, count as usize) {
                if !matches!(entry.FmtValue.CStatus, 0 | 1) || entry.szName.is_null() {
                    continue;
                }
                let Ok(name) = entry.szName.to_string() else {
                    continue;
                };
                diag.valid += 1;
                let Some(engine) =
                    matching_engine(&name, self.luid.HighPart as u32, self.luid.LowPart)
                else {
                    continue;
                };
                let value = entry.FmtValue.Anonymous.doubleValue;
                if value.is_finite() {
                    diag.matching += 1;
                    *engines.entry(engine.to_owned()).or_default() += value.max(0.0);
                }
            }
            // Each instance reports one process on one engine. Sum by engine and use the busiest
            // engine so unrelated activity such as 3D and video decoding is counted separately.
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
        // SAFETY: This instance owns the query exclusively.
        unsafe { PdhCloseQuery(self.query) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn resolves_only_the_selected_adapter() {
        let name = "pid_42_LUID_0X0_0x00AB_phys_0_eng_2_engtype_VideoDecode";
        assert_eq!(
            matching_engine(name, 0, 171),
            Some("phys_0_eng_2_engtype_VideoDecode")
        );
        assert_eq!(matching_engine(name, 1, 171), None);
        assert_eq!(matching_engine(name, 0, 17), None);
        assert_eq!(matching_engine("pid_42_luid_0x0_bad", 0, 0), None);
    }

    #[test]
    fn gpu_ausente_nao_causa_falha() {
        let mut gpu = GpuCollector {
            nvml: None,
            adapters: Vec::new(),
            #[cfg(windows)]
            pdh: None,
            #[cfg(windows)]
            retry_at: None,
            #[cfg(windows)]
            diagnostics: CounterDiagnostics::default(),
        };
        assert!(gpu.collect(true).is_none());
        assert!(gpu.collect(false).is_none());
    }
}
