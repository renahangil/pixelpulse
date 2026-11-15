import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { renderMetrics, type Snapshot } from "./ui/metrics";

// Mude para "click-only" para desativar hover; tempos em milissegundos.
const INTERACTION_MODE = "hover-click" as "hover-click" | "click-only";
const EXPAND_DELAY_MS = 200;
const COLLAPSE_DELAY_MS = 400;

type Mode = "Collapsed" | "Expanded";


// Leva a licença junto no bundle, inclusive ao distribuir somente o executável.
const fontLicense = document.createElement("link");
fontLicense.rel = "license";
fontLicense.href = new URL("./assets/fonts/OFL.txt", import.meta.url).href;
document.head.append(fontLicense);

const island = document.querySelector<HTMLElement>("#island");
const details = document.querySelector<HTMLElement>("#details");
const summary = document.querySelector<HTMLElement>("#summary");
const status = document.querySelector<HTMLParagraphElement>("#status");
const pinState = document.querySelector<HTMLElement>("#pin-state");
const hint = document.querySelector<HTMLParagraphElement>("#hint");
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

function render(): void {
  if (document.hidden || !latest) return;
  renderMetrics(latest, appliedMode === "Expanded");
}
function renderInteraction(): void {
  if (island && island.dataset.mode !== appliedMode) island.dataset.mode = appliedMode;
  if (details) details.hidden = appliedMode !== "Expanded";
  if (summary) summary.hidden = appliedMode === "Expanded";
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
    const glass = await invoke<"acrylic" | "fallback">("get_glass_mode");
    document.documentElement.dataset.glass = glass;
    // Também sincroniza uma eventual recarga da UI com o tamanho inicial da ilha.
    await invoke("set_mode", { mode: "Collapsed" });
    const cached = await invoke<Snapshot | null>("get_snapshot");
    if (!receivedEvent) latest = cached;
    ready = true;
    if (hint) hint.textContent = INTERACTION_MODE === "hover-click"
      ? "Clique fixa. Esc libera." : "Clique abre/fixa. Esc libera.";
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
