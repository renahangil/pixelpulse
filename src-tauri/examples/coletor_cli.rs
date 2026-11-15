//! Execute com `cargo run --release --example coletor_cli`; Ctrl+C encerra.
//! Um número opcional limita as amostras: `cargo run --release --example coletor_cli -- 5`.

use std::{
    error::Error,
    io::{self, Write},
    num::NonZeroUsize,
    thread,
    time::{Duration, Instant},
};

use pixelpulse_lib::collectors::{Collector, Snapshot};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, get_current_pid};

#[derive(Serialize)]
struct ProcessInfo {
    /// Percentual normalizado para a máquina toda, como no Gerenciador de Tarefas.
    cpu_percent: f32,
    ram_used_mb: f64,
}

#[derive(Serialize)]
struct Report {
    snapshot: Snapshot,
    process: ProcessInfo,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let limit = args
        .next()
        .map(|value| value.parse::<NonZeroUsize>())
        .transpose()?;
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "uso: coletor_cli [quantidade de amostras]",
        )
        .into());
    }

    let mut collector = Collector::new()?;
    let mut process_system = System::new()?;
    let pid = get_current_pid()?;
    let process_refresh = ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .without_tasks();
    // Só o nosso PID: não coletamos argumentos, ambiente, disco nem todos os processos.
    process_system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        process_refresh,
    );

    let interval = Duration::from_secs(2);
    let mut next_sample = Instant::now() + interval;
    let mut count = 0;
    let mut output = io::stdout().lock();
    loop {
        // Não fazemos espera ocupada nem tentamos compensar atrasos com rajadas de coleta.
        thread::sleep(next_sample.saturating_duration_since(Instant::now()));
        let started = Instant::now();
        let snapshot = collector.collect()?;
        process_system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            process_refresh,
        );
        let process = process_system.process(pid).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "processo do CLI não encontrado")
        })?;
        let report = Report {
            snapshot,
            process: ProcessInfo {
                cpu_percent: process.cpu_usage() / collector.logical_cpu_count() as f32,
                ram_used_mb: process.memory() as f64 / 1_000_000.0,
            },
        };
        serde_json::to_writer(&mut output, &report)?;
        writeln!(output)?;
        output.flush()?;
        count += 1;
        if limit.is_some_and(|limit| count >= limit.get()) {
            break;
        }
        next_sample = started + interval;
    }
    Ok(())
}
