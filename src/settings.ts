import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t, useLanguage, type Language, type TextKey } from "./ui/i18n";
import type { Configuration, Gpu, Preferences } from "./ui/configuration";

type Section = "diagnostics" | "devices" | "startup" | "language" | "version";
const panel = document.querySelector<HTMLElement>("#panel")!;
const feedback = document.querySelector<HTMLElement>("#feedback")!;
const navigation = document.querySelectorAll<HTMLButtonElement>("nav button");
let section: Section = "diagnostics";
let config: Configuration | undefined;
let startup: boolean | null = null;
let startupRevision = 0;
let busy = false;

function node<K extends keyof HTMLElementTagNameMap>(tag: K, text?: string): HTMLElementTagNameMap[K] {
  const result = document.createElement(tag);
  if (text !== undefined) result.textContent = text;
  return result;
}

function note(key: TextKey): HTMLParagraphElement {
  const p = node("p", t(key));
  p.className = "note";
  return p;
}

function showError(key?: TextKey): void {
  feedback.hidden = !key;
  feedback.textContent = key ? t(key) : "";
}

function fact(list: HTMLElement, label: string, value: string | number | null): void {
  const row = node("div");
  row.className = "fact";
  row.append(node("dt", label), node("dd", value === null ? t("na") : String(value)));
  list.append(row);
}

function option(label: string, detail: string, checked: boolean, disabled: boolean, change?: (value: boolean) => void): HTMLLabelElement {
  const row = node("label");
  row.className = "device";
  const input = node("input");
  input.type = "checkbox";
  input.checked = checked;
  input.disabled = disabled;
  if (change) input.addEventListener("change", () => change(input.checked));
  const description = node("span");
  description.append(node("strong", label));
  if (detail) description.append(node("small", detail));
  row.append(input, description);
  return row;
}

function gpuRows(gpus: readonly Gpu[]): { gpu: Gpu; name: string; primary: boolean }[] {
  return gpus.flatMap(gpu => Array.from({ length: gpu.physical_count }, (_, index) => ({
    gpu,
    name: gpu.physical_count > 1 ? `${gpu.name} · ${t("linked")} ${index + 1}/${gpu.physical_count}` : gpu.name,
    primary: gpu.primary && index === 0,
  })));
}

function render(): void {
  if (config) useLanguage(config.preferences.language);
  document.title = t("title");
  document.querySelector("nav")?.setAttribute("aria-label", t("title"));
  document.querySelector("#close")?.setAttribute("aria-label", t("close"));
  for (const button of Array.from(navigation)) {
    const key = button.dataset.section as Section;
    const label = button.querySelector("[data-section-label]");
    if (label) label.textContent = t(key);
    if (key === section) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }
  document.querySelector("#section-title")!.textContent = t(section);
  panel.replaceChildren();

  if (section === "version" && config) {
    const list = node("dl");
    const rows = [
      [t("installed"), config.version],
      [t("releaseDate"), config.release_date || t("unreleased")],
      [t("distribution"), t(config.distribution)],
      [t("architecture"), config.inventory?.architecture || t("na")],
      [t("build"), config.build_id || t("na")],
    ];
    for (const [label, value] of rows) fact(list, label, value);
    const copy = node("button", t("copyVersion"));
    copy.type = "button";
    copy.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(rows.map(([label, value]) => label + ": " + value).join("\n"));
        copy.textContent = t("copied");
      } catch { showError("error"); }
    });
    panel.append(list, node("h2", t("releaseNotes")), note("notes110"), copy);
    return;
  }

  if (section === "language") {
    const select = node("select");
    for (const [value, name] of [["en-US", "English (US)"], ["pt-BR", "Português (Brasil)"], ["es", "Español"]]) {
      const entry = node("option", name);
      entry.value = value;
      entry.selected = value === config?.preferences.language;
      select.append(entry);
    }
    select.disabled = busy || !config;
    select.setAttribute("aria-label", t("language"));
    select.addEventListener("change", () => {
      const language = select.value as Language;
      void changePreference("set_language", { language }, "select");
    });
    panel.append(select);
    return;
  }

  if (section === "startup") {
    panel.append(option(t("startupToggle"), t("startupNote"), startup === true, busy || startup === null, enabled => void changeStartup(enabled)));
    return;
  }

  const inventory = config?.inventory;
  if (!inventory || !config) {
    panel.append(note("loading"));
    return;
  }

  if (section === "diagnostics") {
    const list = node("dl");
    fact(list, t("windows"), inventory.os);
    fact(list, t("release"), inventory.os_version);
    fact(list, t("build"), inventory.windows_build);
    fact(list, t("architecture"), inventory.architecture);
    fact(list, "CPU", inventory.cpu);
    fact(list, t("cores"), inventory.physical_cores);
    fact(list, t("threads"), inventory.logical_cores);
    fact(list, "RAM", new Intl.NumberFormat(document.documentElement.lang, {
      maximumFractionDigits: 1, style: "unit", unit: "gigabyte",
    }).format(inventory.ram_mb / 1000));
    fact(list, t("system"), inventory.system_volume);
    panel.append(list, node("h2", t("graphics")));
    if (inventory.gpus.length === 0) panel.append(note("emptyGpu"));
    for (const { gpu, name, primary } of gpuRows(inventory.gpus)) {
      const gpuList = node("dl");
      fact(gpuList, primary ? t("primary") : gpu.vendor, name);
      panel.append(gpuList);
    }
    panel.append(node("h2", t("storage")));
    if (inventory.volumes.length === 0) panel.append(note("emptyDisk"));
    const volumes = node("dl");
    for (const volume of inventory.volumes) fact(volumes, volume.letter, volume.label || t("disk"));
    panel.append(volumes);
    return;
  }

  panel.append(note("devicesNote"));
  panel.append(option("CPU", inventory.cpu || t("na"), true, true));
  panel.append(option("RAM", t("selected"), true, true));
  const hasGpu = inventory.gpus.some(gpu => gpu.primary);
  if (!hasGpu) panel.append(option("GPU", t("na"), true, true));
  for (const { gpu, name, primary } of gpuRows(inventory.gpus)) {
    const description = gpu.integrated === null ? t("unknown") : t(gpu.integrated ? "integrated" : "dedicated");
    panel.append(option(name, primary ? `GPU · ${t("primary")} · ${description}` : description, primary, true));
  }
  panel.append(node("h2", t("storage")));
  if (!inventory.volumes.some(volume => volume.system)) {
    panel.append(option(inventory.system_volume || t("disk"), t("na"), true, true));
  }
  const selectedCount = inventory.volumes.filter(volume => !volume.system && config?.preferences.volumes.includes(volume.id)).length;
  for (const volume of inventory.volumes) {
    const label = volume.label ? `${volume.letter}  ${volume.label}` : volume.letter;
    const media = volume.media === "unknown" ? t("unknown") : volume.media.toUpperCase();
    const selected = volume.system || config.preferences.volumes.includes(volume.id);
    const disabled = volume.system || busy || (!selected && selectedCount >= config.volume_limit);
    const row = option(label, volume.system ? `${t("system")} · ${media}` : media, selected, disabled, checked => {
      void changePreference("set_volume", { id: volume.id, selected: checked }, `input[data-volume="${CSS.escape(volume.id)}"]`);
    });
    row.querySelector("input")!.dataset.volume = volume.id;
    panel.append(row);
  }
  if (selectedCount >= config.volume_limit && inventory.volumes.some(volume => !volume.system)) panel.append(note("limit"));
}

async function changePreference(command: string, args: Record<string, unknown>, focus: string): Promise<void> {
  if (busy || !config) return;
  busy = true;
  showError();
  render();
  try {
    config.preferences = await invoke<Preferences>(command, args);
  } catch (error) {
    console.error(error);
    showError("error");
  } finally {
    busy = false;
    render();
    panel.querySelector<HTMLElement>(focus)?.focus();
  }
}

async function readStartup(): Promise<void> {
  if (busy) return;
  const revision = ++startupRevision;
  try {
    const actual = await invoke<boolean>("get_startup");
    if (revision !== startupRevision) return;
    startup = actual;
  } catch (error) {
    if (revision !== startupRevision) return;
    console.error(error);
    if (section === "startup") showError("startupError");
  }
  if (section === "startup") render();
}

async function changeStartup(enabled: boolean): Promise<void> {
  if (busy) return;
  // A startup query triggered by focus can finish after the user clicks.
  // Invalidate that query so its result cannot restore an outdated checkbox state.
  startupRevision += 1;
  busy = true;
  showError();
  render();
  let failed = false;
  try {
    startup = await invoke<boolean>("set_startup", { enabled });
    failed = startup !== enabled;
  } catch (error) {
    console.error(error);
    failed = true;
  } finally {
    busy = false;
    if (failed) {
      if (section === "startup") showError("startupError");
      await readStartup();
    } else {
      render();
    }
    if (section === "startup") panel.querySelector<HTMLInputElement>("input")?.focus();
  }
}

async function load(): Promise<void> {
  try { config = await invoke<Configuration>("get_settings"); render(); }
  catch (error) { console.error(error); showError("error"); }
}

for (const button of Array.from(navigation)) button.addEventListener("click", () => {
  section = button.dataset.section as Section;
  showError();
  render();
  if (section === "startup") void readStartup();
});
const close = () => void invoke("close_settings").catch(console.error);
document.querySelector("#close")?.addEventListener("click", close);
window.addEventListener("keydown", event => { if (event.key === "Escape") close(); });
window.addEventListener("focus", () => { if (section === "startup") void readStartup(); });
void listen("inventory-ready", () => { void load(); }).then(() => load()).catch(console.error);
render();
