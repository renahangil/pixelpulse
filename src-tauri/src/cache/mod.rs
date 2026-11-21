//! One lock keeps the latest snapshot and its history consistent.

pub(crate) mod sampler;

use crate::collectors::Snapshot;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    time::Duration,
};

const HISTORY_LIMIT: usize = 60;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    Collapsed = 0,
    Expanded = 1,
}

impl Mode {
    pub fn interval(self) -> Duration {
        match self {
            Self::Collapsed => Duration::from_secs(5),
            Self::Expanded => Duration::from_secs(2),
        }
    }
}

struct SnapshotCache {
    latest: Option<Snapshot>,
    history: VecDeque<Snapshot>,
}

#[derive(Clone)]
pub struct AppState {
    cache: Arc<RwLock<SnapshotCache>>,
    mode: Arc<AtomicU8>,
    stopped: Arc<AtomicBool>,
    ui_visible: Arc<AtomicBool>,
    wake: SyncSender<()>,
    revision: Arc<AtomicU64>,
}

impl AppState {
    pub fn new_with_receiver() -> (Self, Receiver<()>) {
        // Coalesce rapid mode changes into a single pending wake-up.
        let (wake, receiver) = mpsc::sync_channel(1);
        (
            Self {
                cache: Arc::new(RwLock::new(SnapshotCache {
                    latest: None,
                    history: VecDeque::with_capacity(HISTORY_LIMIT),
                })),
                mode: Arc::new(AtomicU8::new(Mode::Collapsed as u8)),
                stopped: Arc::new(AtomicBool::new(false)),
                ui_visible: Arc::new(AtomicBool::new(true)),
                wake,
                revision: Arc::new(AtomicU64::new(0)),
            },
            receiver,
        )
    }

    pub fn snapshot(&self) -> Result<Option<Snapshot>, String> {
        self.cache
            .read()
            .map(|cache| cache.latest.clone())
            .map_err(|_| "não foi possível ler o cache de métricas".to_owned())
    }

    pub fn store(&self, snapshot: &Snapshot) -> Result<(), String> {
        let mut cache = self
            .cache
            .write()
            .map_err(|_| "não foi possível atualizar o cache de métricas".to_owned())?;
        // Evict first so the buffer never temporarily exceeds 60 samples.
        if cache.history.len() == HISTORY_LIMIT {
            cache.history.pop_front();
        }
        cache.history.push_back(snapshot.clone());
        cache.latest = Some(snapshot.clone());
        Ok(())
    }

    pub fn mode(&self) -> Mode {
        // The atomic selects the interval, while the lock protects the collected metrics.
        match self.mode.load(Ordering::Relaxed) {
            1 => Mode::Expanded,
            _ => Mode::Collapsed,
        }
    }

    pub fn set_mode(&self, mode: Mode) -> Result<(), String> {
        if self.mode.swap(mode as u8, Ordering::Relaxed) != mode as u8 {
            self.notify()?;
            #[cfg(debug_assertions)]
            eprintln!("[modo] {mode:?}: intervalo={}s", mode.interval().as_secs());
        }
        Ok(())
    }

    fn notify(&self) -> Result<(), String> {
        match self.wake.try_send(()) {
            Ok(()) | Err(TrySendError::Full(())) => Ok(()),
            Err(TrySendError::Disconnected(())) => Err("thread de amostragem encerrada".to_owned()),
        }
    }

    pub fn set_ui_visible(&self, visible: bool) -> Result<(), String> {
        if self.ui_visible.swap(visible, Ordering::Relaxed) != visible {
            self.notify()?;
        }
        Ok(())
    }

    pub fn request_sample(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
        let _ = self.notify();
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn ui_visible(&self) -> bool {
        self.ui_visible.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        let _ = self.notify();
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::DiskInfo;

    #[test]
    fn historico_mantem_as_ultimas_60_e_leitura_nao_altera_o_cache() {
        let (state, _receiver) = AppState::new_with_receiver();
        assert!(state.snapshot().unwrap().is_none());
        for value in 0..65 {
            state
                .store(&Snapshot {
                    cpu_percent: value as f32,
                    ram_used_mb: 1.0,
                    ram_total_mb: 2.0,
                    ram_percent: 50.0,
                    disk: Some(DiskInfo {
                        used_percent: 50.0,
                        used_gb: 1.0,
                    }),
                    system_volume: Some("C:\\".to_owned()),
                    volumes: Vec::new(),
                    gpu: None,
                })
                .unwrap();
        }
        let cache = state.cache.read().unwrap();
        assert_eq!(cache.history.len(), 60);
        assert_eq!(cache.history.front().unwrap().cpu_percent, 5.0);
        assert_eq!(cache.history.back().unwrap().cpu_percent, 64.0);
        drop(cache);
        let mut copy = state.snapshot().unwrap().unwrap();
        copy.cpu_percent = 0.0;
        assert_eq!(copy.cpu_percent, 0.0);
        assert_eq!(state.snapshot().unwrap().unwrap().cpu_percent, 64.0);
    }

    #[test]
    fn modo_acorda_a_thread_e_nao_acumula_avisos() {
        let (state, receiver) = AppState::new_with_receiver();
        assert_eq!(state.mode().interval(), Duration::from_secs(5));
        let expanded: Mode = serde_json::from_str("\"Expanded\"").unwrap();
        state.set_mode(expanded).unwrap();
        state.set_mode(Mode::Collapsed).unwrap();
        state.set_mode(expanded).unwrap();
        assert_eq!(state.mode().interval(), Duration::from_secs(2));
        assert!(receiver.try_recv().is_ok());
        assert!(receiver.try_recv().is_err());
        state.stop();
        assert!(state.is_stopped());
        assert!(receiver.try_recv().is_ok());
    }
}
