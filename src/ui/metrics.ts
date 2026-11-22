import type { Volume } from "./configuration";
import { t } from "./i18n";

type DiskUsage = { used_percent: number; used_gb: number };
export interface Snapshot {
  cpu_percent: number;
  ram_used_mb: number;
  ram_total_mb: number;
  ram_percent: number;
  disk: DiskUsage | null;
  system_volume: string | null;
  volumes: { id: string; letter: string; usage: DiskUsage | null }[];
  gpu: {
    integrated: boolean | null;
    usage_percent: number | null;
    vram_used_mb: number | null;
    vram_total_mb: number | null;
    temperature_c: number | null;
  } | null;
}

type Metric = "cpu" | "ram" | "gpu" | "ssd";
const METRICS: Metric[] = ["cpu", "ram", "gpu", "ssd"];
const BLOCK_COUNT = 16;

function percent(value: number | null | undefined): number | null {
  return value == null || !Number.isFinite(value) ? null : Math.max(0, Math.min(100, value));
}

function setText(element: HTMLElement | null, text: string): void {
  if (element && element.textContent !== text) element.textContent = text;
}

function setAttribute(element: HTMLElement, name: string, value: string): void {
  if (element.getAttribute(name) !== value) element.setAttribute(name, value);
}

class SegmentedBar {
  private readonly blocks: HTMLSpanElement[];
  private lit = 0;
  private readonly output: HTMLElement | null;

  constructor(private readonly element: HTMLElement, output: HTMLElement | null) {
    this.output = output;
    this.blocks = Array.from({ length: BLOCK_COUNT }, () => {
      const block = document.createElement("span");
      block.className = "block";
      block.setAttribute("aria-hidden", "true");
      return block;
    });
    // Allocate the 16 elements once and reuse them across samples.
    element.append(...this.blocks);
    setAttribute(element, "aria-valuetext", t("na"));
  }

  update(value: number | null): void {
    const next = value === null ? 0 : Math.round(value / 100 * BLOCK_COUNT);
    // Update the difference only. Going from 8 to 10 lights blocks at indices 8 and 9.
    for (let index = Math.min(this.lit, next); index < Math.max(this.lit, next); index++) {
      this.blocks[index].classList.toggle("is-lit", index < next);
    }
    this.lit = next;
    const text = value === null ? "—" : `${Math.round(value)}%`;
    setText(this.output, text);
    if (value === null) {
      if (this.element.hasAttribute("aria-valuenow")) this.element.removeAttribute("aria-valuenow");
    } else {
      setAttribute(this.element, "aria-valuenow", String(Math.round(value)));
    }
    setAttribute(this.element, "aria-valuetext", value === null ? t("na") : text);
  }
}

const bars = new Map<Metric, SegmentedBar>();
for (const name of METRICS) {
  const element = document.getElementById(`${name}-bar`);
  if (element) bars.set(name, new SegmentedBar(element, document.getElementById(`${name}-value`)));
}
const dots = document.querySelectorAll<HTMLSpanElement>(".dots .dot");
const highest = document.getElementById("highest");
const gpuLabel = document.getElementById("gpu-label");
const systemLabel = document.getElementById("ssd-label");
const extra = document.getElementById("extra-metrics");
const extraBars = new Map<string, { row: HTMLElement; bar: SegmentedBar }>();

export function syncVolumeRows(volumes: readonly Volume[]): void {
  if (!extra) return;
  const required = new Set(volumes.map(volume => volume.id));
  for (const [id, entry] of extraBars) {
    if (!required.has(id)) {
      entry.row.remove();
      extraBars.delete(id);
    }
  }
  for (const volume of volumes) {
    let entry = extraBars.get(volume.id);
    if (!entry) {
      const row = document.createElement("div");
      row.className = "metric metric-ssd";
      const icon = systemLabel?.parentElement?.querySelector("svg")?.cloneNode(true);
      const label = document.createElement("span");
      label.className = "pixel-label";
      label.textContent = volume.letter;
      const element = document.createElement("span");
      element.className = "bar";
      element.setAttribute("role", "progressbar");
      element.setAttribute("aria-label", volume.letter);
      element.setAttribute("aria-valuemin", "0");
      element.setAttribute("aria-valuemax", "100");
      const output = document.createElement("output");
      output.className = "pixel-number";
      output.textContent = "—";
      if (icon) row.append(icon);
      row.append(label, element, output);
      entry = { row, bar: new SegmentedBar(element, output) };
      extraBars.set(volume.id, entry);
    }
    extra.append(entry.row);
  }
}

export function renderMetrics(snapshot: Snapshot, expanded: boolean): void {
  const values = [
    percent(snapshot.cpu_percent), percent(snapshot.ram_percent),
    percent(snapshot.gpu?.usage_percent), percent(snapshot.disk?.used_percent),
  ];
  const gpuName = snapshot.gpu?.integrated === true ? "iGPU" : "GPU";
  setText(gpuLabel, gpuName);
  const diskLabel = snapshot.system_volume || t("disk");
  setText(systemLabel, diskLabel);
  if (expanded) {
    for (const [id, entry] of extraBars) {
      const usage = snapshot.volumes?.find(volume => volume.id === id)?.usage;
      entry.bar.update(percent(usage?.used_percent));
    }
    METRICS.forEach((name, index) => bars.get(name)?.update(values[index]));
  } else {
    // Exclude unavailable GPU usage from the maximum so an unknown value stays distinct from zero.
    let winner = -1;
    let maximum = -1;
    values.forEach((value, index) => {
      if (value !== null && value > maximum) {
        winner = index;
        maximum = value;
      }
    });
    dots.forEach((dot, index) => dot.classList.toggle("is-highest", index === winner));
    setText(highest, winner < 0 ? "—" : `${winner === 3 ? diskLabel : winner === 2 ? gpuName : METRICS[winner].toUpperCase()} ${Math.round(maximum)}%`);
  }
}
