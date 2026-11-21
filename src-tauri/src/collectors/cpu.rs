use std::{thread, time::Instant};

use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

pub(super) fn prime(system: &mut System) -> Instant {
    // CPU usage is a counter delta. The first read only establishes the baseline.
    system.refresh_cpu_usage();
    Instant::now()
}

pub(super) fn collect(system: &mut System, last_sample: &mut Instant) -> f32 {
    // Wait for a real sample even when the caller asks for data immediately.
    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL.saturating_sub(last_sample.elapsed()));
    system.refresh_cpu_usage();
    *last_sample = Instant::now();
    system.global_cpu_usage()
}
