import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { renderMetrics, syncVolumeRows, type Snapshot } from "./ui/metrics";
import { t, useLanguage } from "./ui/i18n";
import type { Configuration, Preferences } from "./ui/configuration";

// Use "click-only" to disable hover. All delays are in milliseconds.
const INTERACTION_MODE = "hover-click" as "hover-click" | "click-only";
const EXPAND_DELAY_MS = 200;
const COLLAPSE_DELAY_MS = 400;

type Mode = "Collapsed" | "Expanded";


// Include the font license in the bundle so it ships with the standalone executable.
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
const unlisten: UnlistenFn[] = [];
let configuration: Configuration | undefined;
let currentVolumeCount = 0;
let resizeRevision = 0;
let disposed = false;
let eventRevision = 0;
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
  const label = t(pinned ? "pinned" : "unpinned");
  if (hint) hint.textContent = t(INTERACTION_MODE === "hover-click" ? "hint" : "hintClick");
  island?.setAttribute("aria-label", t("island"));
  summary?.setAttribute("aria-label", t("highest"));
  if (pinState && pinState.textContent !== label) pinState.textContent = label;
  render();
}

async function syncMode(): Promise<void> {
  if (syncing || !ready || disposed) return;
  syncing = true;
  try {
    // Serialize mode requests so a late expand response cannot undo a collapse.
    while (!disposed && appliedMode !== desiredMode) {
      const next = desiredMode;
      resizeRevision += 1;
      await invoke("set_mode", { mode: next });
      appliedMode = next;
      // Read the cache on expansion because the summary may have suppressed some events.
      await readLatest();
      renderInteraction();
    }
    if (status) status.textContent = t("ready");
  } catch (error) {
    console.error(error);
    desiredMode = appliedMode;
    if (status) status.textContent = t("error");
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
island?.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  void invoke("open_context_menu").catch(error => {
    console.error(error);
  });
});
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
    unlisten.push(await listen<Snapshot>("snapshot", ({ payload }) => {
      eventRevision += 1;
      latest = payload;
      render();
    }));
    unlisten.push(await listen<Preferences>("preferences-changed", ({ payload }) => {
      if (!configuration) return;
      configuration.preferences = payload;
      applyConfiguration(true);
    }));
    unlisten.push(await listen("inventory-ready", () => void readConfiguration().catch(console.error)));
    if (disposed) { unlisten.forEach(stop => stop()); return; }
    await readConfiguration();
    const glass = await invoke<"acrylic" | "fallback" | "opaque">("get_glass_mode");
    document.documentElement.dataset.glass = glass;
    // Match the initial native window size after a webview reload.
    await invoke("set_mode", { mode: "Collapsed" });
    await syncVisibility();
    ready = true;
    renderInteraction();
    await syncMode();
  } catch (error) {
    console.error(error);
    if (status) status.textContent = t("error");
  }
}

async function readConfiguration(): Promise<void> {
  configuration = await invoke<Configuration>("get_settings");
  applyConfiguration(false);
}

function applyConfiguration(animate: boolean): void {
  if (!configuration) return;
  const { preferences, inventory, volume_limit } = configuration;
  useLanguage(preferences.language);
  const selected = (inventory?.volumes ?? []).filter(volume => !volume.system && preferences.volumes.includes(volume.id))
    .slice(0, volume_limit);
  syncVolumeRows(selected);
  renderInteraction();
  if (ready && status) status.textContent = t("ready");
  if (animate && selected.length !== currentVolumeCount && appliedMode === "Expanded") {
    void animateHeight(230 + 36 * selected.length).catch(console.error);
  }
  currentVolumeCount = selected.length;
}

async function animateHeight(target: number): Promise<void> {
  const revision = ++resizeRevision;
  const initial = window.innerHeight;
  const duration = matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 155;
  const start = performance.now();
  let last = initial;
  while (!disposed && revision === resizeRevision && appliedMode === "Expanded") {
    const progress = duration === 0 ? 1 : Math.min(1, (performance.now() - start) / duration);
    const height = Math.round(initial + (target - initial) * (1 - (1 - progress) ** 3));
    if (height !== last) {
      await invoke("resize_island", { height });
      last = height;
    }
    if (progress === 1) break;
    await new Promise<number>(resolve => requestAnimationFrame(resolve));
  }
}

async function readLatest(): Promise<void> {
  const revision = eventRevision;
  const cached = await invoke<Snapshot | null>("get_snapshot");
  // Ignore a cached response if a newer event has already rendered.
  if (revision === eventRevision) latest = cached;
  render();
}

let visibilitySync: Promise<void> = Promise.resolve();
function syncVisibility(): Promise<void> {
  const visible = !document.hidden;
  // Serialize visibility updates so a late hide response cannot undo restoration.
  visibilitySync = visibilitySync.catch(() => {}).then(async () => {
    await invoke("set_ui_visibility", { visible });
    if (visible && !disposed) await readLatest();
  });
  return visibilitySync;
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) clearTimers();
  void syncVisibility().catch(error => {
    console.error(error);
    if (status) status.textContent = t("error");
  });
});
window.addEventListener("pagehide", () => {
  disposed = true;
  clearTimers();
  resizeRevision += 1;
  unlisten.forEach(stop => stop());
}, { once: true });
void start();
