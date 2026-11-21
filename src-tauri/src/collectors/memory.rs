use sysinfo::{MemoryRefreshKind, System};

use super::BYTES_PER_MB;

pub(super) fn collect(system: &mut System) -> (f64, f64, f32) {
    // The island displays physical RAM, so skip swap.
    system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
    from_bytes(system.used_memory(), system.total_memory())
}

fn from_bytes(used: u64, total: u64) -> (f64, f64, f32) {
    let used = used.min(total);
    // Handle an unknown total before dividing or serializing to JSON.
    let percent = if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    };
    (
        used as f64 / BYTES_PER_MB,
        total as f64 / BYTES_PER_MB,
        percent,
    )
}

#[cfg(test)]
mod tests {
    use super::from_bytes;

    #[test]
    fn ram_converte_unidades_e_trata_total_zero() {
        assert_eq!(
            from_bytes(500_000_000, 2_000_000_000),
            (500.0, 2000.0, 25.0)
        );
        assert_eq!(from_bytes(0, 0), (0.0, 0.0, 0.0));
    }
}
