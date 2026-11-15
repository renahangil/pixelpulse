use super::AppState;
use crate::collectors::Collector;
use std::{
    io,
    sync::{
        Mutex,
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use tauri::{AppHandle, Emitter};

pub(crate) struct Sampler {
    state: AppState,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Sampler {
    pub fn start(
        app: AppHandle,
        state: AppState,
        changes: Receiver<()>,
        mut collector: Collector,
    ) -> io::Result<Self> {
        let worker_state = state.clone();
        let worker = thread::Builder::new().name("pixelpulse-sampler".to_owned()).spawn(move || {
            let mut count = 0_u64;
            let mut previous_event: Option<Instant> = None;
            while !worker_state.is_stopped() {
                let started = Instant::now();
                match collector.collect() {
                    Ok(snapshot) => {
                        if let Err(error) = worker_state.store(&snapshot) {
                            eprintln!("[amostragem] {error}");
                            break;
                        }
                        // O lock já foi liberado: serialização/evento não bloqueiam get_snapshot.
                        match app.emit("snapshot", &snapshot) {
                            Ok(()) => {
                                count += 1;
                                let now = Instant::now();
                                let delta = previous_event.map(|last| now.duration_since(last).as_secs_f64());
                                previous_event = Some(now);
                                // Contador temporário, sem gravar arquivos nem adicionar crate de log.
                                eprintln!("[snapshot #{count}] modo={:?} intervalo={}s delta={delta:?}", worker_state.mode(), worker_state.mode().interval().as_secs());
                            }
                            Err(error) => eprintln!("[snapshot] falha ao emitir: {error}"),
                        }
                    }
                    Err(error) => eprintln!("[amostragem] falha na coleta: {error}"),
                }
                if !wait_for_next(&worker_state, &changes, started) { break; }
            }
        })?;
        Ok(Self {
            state,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub fn shutdown(&self) {
        self.state.stop();
        match self.worker.lock() {
            Ok(mut worker) => {
                if let Some(worker) = worker.take()
                    && worker.join().is_err()
                {
                    eprintln!("[amostragem] thread encerrada por panic");
                }
            }
            Err(_) => eprintln!("[amostragem] não foi possível aguardar o encerramento"),
        }
    }
}

fn wait_for_next(state: &AppState, changes: &Receiver<()>, last_sample: Instant) -> bool {
    loop {
        if state.is_stopped() {
            return false;
        }
        let remaining = state
            .mode()
            .interval()
            .saturating_sub(last_sample.elapsed());
        if remaining.is_zero() {
            return true;
        }
        // Dorme até o prazo OU até uma mudança de modo/saída; não existe loop ocupado.
        match changes.recv_timeout(remaining) {
            Ok(()) => continue,
            // Recalcular o prazo também trata uma mudança de modo junto ao timeout.
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}
