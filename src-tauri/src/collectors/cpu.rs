use std::{thread, time::Instant};

use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

pub(super) fn prime(system: &mut System) -> Instant {
    // Uso de CPU é uma diferença de contadores; a primeira leitura só cria a base.
    system.refresh_cpu_usage();
    Instant::now()
}

pub(super) fn collect(system: &mut System, last_sample: &mut Instant) -> f32 {
    // Mesmo uma chamada imediata ao coletor não publica o zero inicial como medição.
    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL.saturating_sub(last_sample.elapsed()));
    system.refresh_cpu_usage();
    *last_sample = Instant::now();
    system.global_cpu_usage()
}
