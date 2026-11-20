import type { Language } from "./i18n";

export interface Preferences {
  language: Language;
  volumes: string[];
}

export interface Volume {
  id: string;
  letter: string;
  label: string;
  media: "ssd" | "hdd" | "unknown";
  system: boolean;
}

export interface Gpu {
  name: string;
  vendor: string;
  integrated: boolean | null;
  primary: boolean;
  physical_count: number;
}

export interface Inventory {
  os: string | null;
  os_version: string | null;
  windows_build: string | null;
  architecture: string;
  cpu: string | null;
  physical_cores: number | null;
  logical_cores: number;
  ram_mb: number;
  system_volume: string | null;
  volumes: Volume[];
  gpus: Gpu[];
}

export interface Configuration {
  preferences: Preferences;
  inventory: Inventory | null;
  volume_limit: number;
}
