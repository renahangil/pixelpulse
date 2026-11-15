import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Mude para "click-only" para desativar hover; tempos em milissegundos.
const INTERACTION_MODE = "hover-click" as "hover-click" | "click-only";
const EXPAND_DELAY_MS = 200;
const COLLAPSE_DELAY_MS = 400;

type Mode = "Collapsed" | "Expanded";
interface Snapshot {
  cpu_percent: number;
  ram_used_mb: number;
  ram_total_mb: number;
  ram_percent: number;
  disk: { used_percent: number; used_gb: number };
  gpu: {
    usage_percent: number | null;
    vram_used_mb: number | null;
    vram_total_mb: number | null;
    temperature_c: number | null;
  } | null;
}

const island = document.querySelector<HTMLElement>("#island");
const details = document.querySelector<HTMLElement>("#details");
const metrics = document.querySelector<HTMLPreElement>("#metrics");
const status = document.querySelector<HTMLParagraphElement>("#status");
const pinState = document.querySelector<HTMLElement>("#pin-state");
const hint = document.querySelector<HTMLParagraphElement>("#hint");
const formatter = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 2 });
let latest: Snapshot | null = null;
let unlisten: UnlistenFn | undefined;
let disposed = false;
let receivedEvent = false;
let ready = false;
let syncing = false;
let appliedMode: Mode = "Collapsed";
let desiredMode: Mode = "Collapsed";
let pinned = false;
let inside = false;
let expandTimer: number | undefined;
let collapseTimer: number | undefined;

function value(number: number | null | undefined, unit: string): string {
  return number == null ? "N/D" : `${formatter.format(number)}${unit}`;
}

function render(): void {
  if (document.hidden || appliedMode !== "Expanded" || !latest || !metrics) return;
  const s = latest;
  const text = [
    `CPU: ${value(s.cpu_percent, "%")}`,
    `RAM: ${value(s.ram_used_mb, "")} / ${value(s.ram_total_mb, " MB")} (${value(s.ram_percent, "%")})`,
    `Disco: ${value(s.disk.used_percent, "%")} — ${value(s.disk.used_gb, " GB")} usados`,
    `GPU: ${value(s.gpu?.usage_percent, "%")}`,
    `VRAM: ${value(s.gpu?.vram_used_mb, "")} / ${value(s.gpu?.vram_total_mb, " MB")}`,
    `Temperatura GPU: ${value(s.gpu?.temperature_c, " °C")}`,
  ].join("\n");
  if (metrics.textContent !== text) metrics.textContent = text;
}

function renderInteraction(): void {
  if (island && island.dataset.mode !== appliedMode) island.dataset.mode = appliedMode;
  if (details) details.hidden = appliedMode !== "Expanded";
  const label = pinned ? "Fixada" : "Livre";
  if (pinState && pinState.textContent !== label) pinState.textContent = label;
  render();
}

async function syncMode(): Promise<void> {
  if (syncing || !ready || disposed) return;
  syncing = true;
  try {
    // Um comando por vez: uma resposta lenta de expandir não desfaz um recolher posterior.
    while (!disposed && appliedMode !== desiredMode) {
      const next = desiredMode;
      await invoke("set_mode", { mode: next });
      appliedMode = next;
      renderInteraction();
    }
    if (status) status.textContent = "Recebendo métricas.";
  } catch (error) {
    desiredMode = appliedMode;
    if (status) status.textContent = `Erro ao ajustar janela: ${String(error)}`;
  } finally {
    syncing = false;
  }
}

function requestMode(mode: Mode): void {
  desiredMode = mode;
  void syncMode();
}

function clearTimers(): void {
  window.clearTimeout(expandTimer);
  window.clearTimeout(collapseTimer);
  expandTimer = undefined;
  collapseTimer = undefined;
}

function followPointer(): void {
  requestMode(pinned || (INTERACTION_MODE === "hover-click" && inside) ? "Expanded" : "Collapsed");
}

function togglePin(): void {
  pinned = !pinned;
  clearTimers();
  renderInteraction();
  followPointer();
}

island?.addEventListener("mouseenter", () => {
  inside = true;
  clearTimers();
  if (INTERACTION_MODE === "hover-click" && !pinned && desiredMode === "Collapsed") {
    expandTimer = window.setTimeout(() => {
      expandTimer = undefined;
      if (inside && !pinned) requestMode("Expanded");
    }, EXPAND_DELAY_MS);
  }
});

island?.addEventListener("mouseleave", () => {
  inside = false;
  clearTimers();
  if (INTERACTION_MODE === "hover-click" && !pinned) {
    collapseTimer = window.setTimeout(() => {
      collapseTimer = undefined;
      if (!inside && !pinned) requestMode("Collapsed");
    }, COLLAPSE_DELAY_MS);
  }
});

island?.addEventListener("click", togglePin);
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    pinned = false;
    clearTimers();
    renderInteraction();
    followPointer();
  } else if ((event.key === "Enter" || event.key === " ") && event.target === island) {
    event.preventDefault();
    togglePin();
  }
});

async function start(): Promise<void> {
  try {
    unlisten = await listen<Snapshot>("snapshot", ({ payload }) => {
      receivedEvent = true;
      latest = payload;
      render();
    });
    if (disposed) { unlisten(); return; }
    // Também sincroniza uma eventual recarga da UI com o tamanho inicial da ilha.
    await invoke("set_mode", { mode: "Collapsed" });
    const cached = await invoke<Snapshot | null>("get_snapshot");
    if (!receivedEvent) latest = cached;
    ready = true;
    if (hint) hint.textContent = INTERACTION_MODE === "hover-click"
      ? "Passe o mouse; clique fixa. Esc libera." : "Clique abre/fixa; novo clique ou Esc libera.";
    renderInteraction();
    await syncMode();
  } catch (error) {
    if (status) status.textContent = `Erro ao conectar: ${String(error)}`;
  }
}

document.addEventListener("visibilitychange", render);
window.addEventListener("pagehide", () => {
  disposed = true;
  clearTimers();
  unlisten?.();
}, { once: true });
void start();
