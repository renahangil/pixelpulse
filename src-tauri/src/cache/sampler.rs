use super::{AppState, Mode};
use crate::{
    collectors::{Collector, Snapshot},
    settings::Settings,
};
use std::{
    io,
    sync::{
        Mutex,
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

pub(crate) struct Sampler {
    state: AppState,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Sampler {
    pub fn start(app: AppHandle, state: AppState, changes: Receiver<()>) -> io::Result<Self> {
        let window = app.get_webview_window("main").ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "janela principal não encontrada")
        })?;
        // Guardar só o identificador evita getters síncronos no event loop da UI:
        // ao encerrar, a UI pode aguardar join sem prender a thread em um getter.
        #[cfg(target_os = "windows")]
        let hwnd = window.hwnd().map_err(io::Error::other)?.0 as usize;
        let worker_state = state.clone();
        let worker = thread::Builder::new()
            .name("pixelpulse-sampler".to_owned())
            .spawn(move || {
                let mut collector = match Collector::new() {
                    Ok(collector) => collector,
                    Err(error) => {
                        eprintln!("[inventory] {error}");
                        return;
                    }
                };
                let settings = app.state::<Settings>();
                let inventory = collector.inventory();
                if let Ok(mut slot) = settings.inventory.write() {
                    *slot = Some(inventory);
                }
                let _ = app.emit("inventory-ready", ());
                let mut previous = None;
                let mut paused = false;
                #[cfg(debug_assertions)]
                let mut count = 0_u64;
                while !worker_state.is_stopped() {
                    // Falta de foco não significa ocultação: a ilha nasce sem foco.
                    #[cfg(target_os = "windows")]
                    let native_visible = {
                        use windows::Win32::{
                            Foundation::HWND,
                            UI::WindowsAndMessaging::{IsIconic, IsWindowVisible},
                        };
                        // SAFETY: consulta somente flags da nossa janela; se destruída,
                        // um HWND inválido retorna false. Nenhum ponteiro é desreferenciado aqui.
                        unsafe {
                            IsWindowVisible(HWND(hwnd as _)).as_bool()
                                && !IsIconic(HWND(hwnd as _)).as_bool()
                        }
                    };
                    #[cfg(not(target_os = "windows"))]
                    let native_visible = window.is_visible().unwrap_or(false)
                        && !window.is_minimized().unwrap_or(true);
                    if !worker_state.ui_visible() || !native_visible {
                        paused = true;
                        previous = None; // Ao voltar, enviar uma leitura mesmo se o valor repetir.
                        // O WebView avisa visibilitychange. A checagem nativa a cada 5 s
                        // cobre hide/show sem esse aviso; aqui não há coleta nem eventos.
                        let resumed = if worker_state.ui_visible() {
                            !matches!(
                                changes.recv_timeout(Duration::from_secs(5)),
                                Err(RecvTimeoutError::Disconnected)
                            )
                        } else {
                            changes.recv().is_ok()
                        };
                        if !resumed {
                            break;
                        }
                        continue;
                    }
                    if paused {
                        collector.reset_cpu_baseline();
                        paused = false;
                    }
                    let started = Instant::now();
                    let revision = worker_state.revision();
                    let preferences = settings.preferences();
                    match collector.collect_for_ui(&preferences.volumes) {
                        Ok(snapshot) => {
                            if let Err(error) = worker_state.store(&snapshot) {
                                eprintln!("[amostragem] {error}");
                                break;
                            }
                            // O histórico recebe toda amostra; só o evento usa a projeção visual.
                            // Não comparar só o inteiro: um segmento pode mudar antes do texto.
                            let presentation = Presentation::new(&snapshot, worker_state.mode());
                            if worker_state.ui_visible() && previous.as_ref() != Some(&presentation)
                            {
                                match app.emit("snapshot", &snapshot) {
                                    Ok(()) => {
                                        previous = Some(presentation);
                                        #[cfg(debug_assertions)]
                                        {
                                            count += 1;
                                            eprintln!(
                                                "[snapshot #{count}] modo={:?}",
                                                worker_state.mode()
                                            );
                                        }
                                    }
                                    Err(error) => eprintln!("[snapshot] falha ao emitir: {error}"),
                                }
                            }
                        }
                        Err(error) => eprintln!("[amostragem] falha na coleta: {error}"),
                    }
                    if !wait_for_next(&worker_state, &changes, started, revision) {
                        break;
                    }
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

fn wait_for_next(
    state: &AppState,
    changes: &Receiver<()>,
    last_sample: Instant,
    revision: u64,
) -> bool {
    let mode = state.mode();
    loop {
        if state.is_stopped() {
            return false;
        }
        if !state.ui_visible() || state.mode() != mode || state.revision() != revision {
            return true;
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

/// Mesmas quantizações usadas pelo TypeScript, sem strings nem alocações por evento.
#[derive(Debug, PartialEq, Eq)]
enum Presentation {
    Collapsed(Option<(usize, u8)>),
    Expanded([Option<(u8, u8)>; 4], Vec<(String, Option<(u8, u8)>)>),
}

impl Presentation {
    fn new(snapshot: &Snapshot, mode: Mode) -> Self {
        let values = [
            Some(snapshot.cpu_percent),
            Some(snapshot.ram_percent),
            snapshot
                .gpu
                .as_ref()
                .and_then(|gpu| gpu.usage_percent)
                .map(|value| value as f32),
            snapshot.disk.as_ref().map(|disk| disk.used_percent),
        ]
        .map(|value| {
            value
                .filter(|value| value.is_finite())
                .map(|value| value.clamp(0.0, 100.0))
        });
        match mode {
            Mode::Collapsed => Self::Collapsed(
                values
                    .iter()
                    .enumerate()
                    .fold(None, |winner, (index, value)| match value {
                        Some(value) if winner.is_none_or(|(_, maximum)| *value > maximum) => {
                            Some((index, *value))
                        }
                        _ => winner,
                    })
                    .map(|(index, value)| (index, value.round() as u8)),
            ),
            Mode::Expanded => Self::Expanded(
                values.map(|value| value.map(bar)),
                snapshot
                    .volumes
                    .iter()
                    .map(|volume| {
                        (
                            volume.id.clone(),
                            volume.usage.as_ref().map(|usage| bar(usage.used_percent)),
                        )
                    })
                    .collect(),
            ),
        }
    }
}

fn bar(value: f32) -> (u8, u8) {
    (value.round() as u8, (value / 100.0 * 16.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::DiskInfo;

    #[test]
    fn deduplicacao_respeita_texto_segmentos_modo_e_gpu_indisponivel() {
        let mut snapshot = Snapshot {
            cpu_percent: 3.1,
            ram_used_mb: 1.0,
            ram_total_mb: 2.0,
            ram_percent: 50.0,
            disk: Some(DiskInfo {
                used_percent: 25.0,
                used_gb: 1.0,
            }),
            system_volume: Some("C:\\".to_owned()),
            volumes: Vec::new(),
            gpu: None,
        };
        let collapsed = Presentation::new(&snapshot, Mode::Collapsed);
        let expanded = Presentation::new(&snapshot, Mode::Expanded);
        snapshot.cpu_percent = 3.2; // Texto continua 3%, mas o primeiro segmento acende.
        assert_eq!(collapsed, Presentation::new(&snapshot, Mode::Collapsed));
        assert_ne!(expanded, Presentation::new(&snapshot, Mode::Expanded));
        snapshot.cpu_percent = 3.21;
        assert_eq!(
            Presentation::new(&snapshot, Mode::Expanded),
            Presentation::Expanded(
                [Some((3, 1)), Some((50, 8)), None, Some((25, 4))],
                Vec::new()
            )
        );
        assert_ne!(collapsed, Presentation::new(&snapshot, Mode::Expanded));

        // O percentual continua 50%, mas agora o ponto e o rótulo são da CPU.
        snapshot.cpu_percent = 50.0;
        assert_eq!(
            Presentation::new(&snapshot, Mode::Collapsed),
            Presentation::Collapsed(Some((0, 50)))
        );
        assert_ne!(collapsed, Presentation::new(&snapshot, Mode::Collapsed));
    }

    #[test]
    fn ocultar_interrompe_espera_e_parar_acorda_thread() {
        let (state, changes) = AppState::new_with_receiver();
        state.set_ui_visible(false).unwrap();
        assert!(wait_for_next(
            &state,
            &changes,
            Instant::now(),
            state.revision()
        ));
        assert!(!state.ui_visible());
        assert!(changes.try_recv().is_ok());
        let other = state.clone();
        let worker = thread::spawn(move || {
            changes.recv().unwrap();
            assert!(other.is_stopped());
        });
        state.stop();
        worker.join().unwrap();
    }
}
