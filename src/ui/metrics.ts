export interface Snapshot {
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

  constructor(private readonly element: HTMLElement, name: Metric) {
    this.output = document.getElementById(`${name}-value`);
    this.blocks = Array.from({ length: BLOCK_COUNT }, () => {
      const block = document.createElement("span");
      block.className = "block";
      block.setAttribute("aria-hidden", "true");
      return block;
    });
    // Alocamos os 16 elementos uma vez, não em cada evento de métricas.
    element.append(...this.blocks);
    setAttribute(element, "aria-valuetext", "N/D");
  }

  update(value: number | null): void {
    const next = value === null ? 0 : Math.round(value / 100 * BLOCK_COUNT);
    // Só atravessamos a diferença: de 8 para 10 acende apenas os blocos 8 e 9.
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
    setAttribute(this.element, "aria-valuetext", value === null ? "N/D" : text);
  }
}

const bars = new Map<Metric, SegmentedBar>();
for (const name of METRICS) {
  const element = document.getElementById(`${name}-bar`);
  if (element) bars.set(name, new SegmentedBar(element, name));
}
const dots = document.querySelectorAll<HTMLSpanElement>(".dots .dot");
const highest = document.getElementById("highest");

export function renderMetrics(snapshot: Snapshot, expanded: boolean): void {
  const values = [
    percent(snapshot.cpu_percent), percent(snapshot.ram_percent),
    percent(snapshot.gpu?.usage_percent), percent(snapshot.disk.used_percent),
  ];
  if (expanded) {
    METRICS.forEach((name, index) => bars.get(name)?.update(values[index]));
  } else {
    // GPU indisponível não vira 0%; ela simplesmente não participa do máximo.
    let winner = -1;
    let maximum = -1;
    values.forEach((value, index) => {
      if (value !== null && value > maximum) {
        winner = index;
        maximum = value;
      }
    });
    dots.forEach((dot, index) => dot.classList.toggle("is-highest", index === winner));
    setText(highest, winner < 0 ? "—" : `${METRICS[winner].toUpperCase()} ${Math.round(maximum)}%`);
  }
}
