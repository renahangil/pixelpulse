import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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

const metrics = document.querySelector<HTMLPreElement>("#metrics");
const status = document.querySelector<HTMLParagraphElement>("#status");
const modeLabel = document.querySelector<HTMLParagraphElement>("#mode");
const collapsed = document.querySelector<HTMLButtonElement>("#collapsed");
const expanded = document.querySelector<HTMLButtonElement>("#expanded");
const formatter = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 2 });
let latest: Snapshot | null = null;
let unlisten: UnlistenFn | undefined;
let disposed = false;
let receivedEvent = false;

function value(number: number | null | undefined, unit: string): string {
  return number == null ? "N/D" : `${formatter.format(number)} ${unit}`;
}

function render(): void {
  if (document.hidden || !latest || !metrics) return;
  const s = latest;
  const text = [
    `CPU: ${value(s.cpu_percent, "%")}`,
    `RAM: ${value(s.ram_used_mb, "MB")} / ${value(s.ram_total_mb, "MB")} (${value(s.ram_percent, "%")})`,
    `Disco do sistema: ${value(s.disk.used_percent, "%")} — ${value(s.disk.used_gb, "GB")} usados`,
    `GPU: ${value(s.gpu?.usage_percent, "%")}`,
    `VRAM: ${value(s.gpu?.vram_used_mb, "MB")} / ${value(s.gpu?.vram_total_mb, "MB")}`,
    `Temperatura GPU: ${value(s.gpu?.temperature_c, "°C")}`,
  ].join("\n");
  // Eventos continuam chegando quando oculto; o DOM só recebe texto visível e novo.
  if (metrics.textContent !== text) metrics.textContent = text;
}

function buttonsDisabled(disabled: boolean): void {
  if (collapsed) collapsed.disabled = disabled;
  if (expanded) expanded.disabled = disabled;
}

async function setMode(mode: Mode): Promise<void> {
  buttonsDisabled(true);
  try {
    await invoke("set_mode", { mode });
    if (modeLabel) modeLabel.textContent = mode === "Collapsed"
      ? "Modo: Recolhido (5 s)" : "Modo: Expandido (2 s)";
    if (status) status.textContent = "Recebendo eventos snapshot.";
  } catch (error) {
    if (status) status.textContent = `Erro ao trocar o modo: ${String(error)}`;
  } finally {
    if (!disposed) buttonsDisabled(false);
  }
}

async function start(): Promise<void> {
  try {
    // Ouvir antes de consultar o cache evita perder eventos durante a inicialização.
    unlisten = await listen<Snapshot>("snapshot", ({ payload }) => {
      receivedEvent = true;
      latest = payload;
      render();
    });
    if (disposed) { unlisten(); return; }
    const cached = await invoke<Snapshot | null>("get_snapshot");
    // Uma resposta inicial mais antiga não pode sobrescrever um evento já recebido.
    if (!receivedEvent) latest = cached;
    render();
    if (status) status.textContent = "Recebendo eventos snapshot.";
    buttonsDisabled(false);
  } catch (error) {
    if (status) status.textContent = `Erro ao conectar: ${String(error)}`;
  }
}

collapsed?.addEventListener("click", () => { void setMode("Collapsed"); });
expanded?.addEventListener("click", () => { void setMode("Expanded"); });
document.addEventListener("visibilitychange", render);
window.addEventListener("pagehide", () => {
  disposed = true;
  unlisten?.();
}, { once: true });
void start();
