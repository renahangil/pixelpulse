use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::collectors::Inventory;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en-US")]
    English,
    #[serde(rename = "pt-BR")]
    Portuguese,
    #[serde(rename = "es")]
    Spanish,
}

impl Language {
    pub fn menu(self) -> (&'static str, &'static str) {
        static CATALOG: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
        let catalog = CATALOG.get_or_init(|| {
            serde_json::from_str(
                include_str!("../../src/ui/locales.json").trim_start_matches('\u{feff}'),
            )
            .unwrap_or_default()
        });
        let locale = match self {
            Self::English => "en-US",
            Self::Portuguese => "pt-BR",
            Self::Spanish => "es",
        };
        (
            catalog[locale]["title"]
                .as_str()
                .unwrap_or("System Diagnostics"),
            catalog[locale]["exit"]
                .as_str()
                .unwrap_or("Exit PixelPulse"),
        )
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Preferences {
    pub language: Language,
    pub volumes: Vec<String>,
}

pub struct Settings {
    path: Option<PathBuf>,
    prefs: Mutex<Preferences>,
    pub inventory: RwLock<Option<Inventory>>,
}

impl Settings {
    pub fn new(home: Option<&Path>) -> Self {
        let path = home.map(|home| home.join(".pixelpulse").join("settings.json"));
        let prefs: Preferences = path
            .as_ref()
            .and_then(|path| match fs::read(path) {
                Ok(data) => serde_json::from_slice(&data)
                    .inspect_err(|error| eprintln!("[settings] {error}"))
                    .ok(),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => {
                    eprintln!("[settings] {error}");
                    None
                }
            })
            .unwrap_or_default();

        Self {
            path,
            prefs: Mutex::new(prefs),
            inventory: RwLock::new(None),
        }
    }

    pub fn preferences(&self) -> Preferences {
        self.prefs
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn visible_volumes(&self) -> usize {
        let selected = self.preferences().volumes;
        self.inventory
            .read()
            .ok()
            .and_then(|inventory| {
                inventory.as_ref().map(|inventory| {
                    inventory
                        .volumes
                        .iter()
                        .filter(|volume| !volume.system && selected.contains(&volume.id))
                        .count()
                })
            })
            .unwrap_or_default()
    }

    pub fn update(
        &self,
        apply: impl FnOnce(&mut Preferences) -> Result<(), String>,
    ) -> Result<Preferences, String> {
        let mut lock = self.prefs.lock().map_err(|error| error.to_string())?;
        let mut next = lock.clone();
        apply(&mut next)?;
        let path = self.path.as_ref().ok_or("User profile unavailable")?;
        save(path, &next).map_err(|error| error.to_string())?;
        *lock = next.clone();
        Ok(next)
    }
}

fn save(path: &Path, preferences: &Preferences) -> io::Result<()> {
    let folder = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(folder)?;
    let temp = folder.join(format!("settings-{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::File::create(&temp)?;
        serde_json::to_writer(&mut file, preferences)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        // Renaming within the same directory preserves the previous file if writing fails.
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
