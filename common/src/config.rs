use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::APP_ID;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub folders: Vec<PathBuf>,
}

impl Config {
    fn file_path() -> Option<PathBuf> {
        dirs::config_dir()
            .or_else(dirs::config_local_dir)
            .map(|path| path.join(APP_ID).join("config.json"))
    }

    pub fn load() -> Self {
        Self::file_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::file_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not determine the config directory for this OS",
            )
        })?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json =
            serde_json::to_string_pretty(self).expect("Config serializes to JSON without errors");

        std::fs::write(path, json)
    }

    pub fn add_folder(&mut self, folder: PathBuf) {
        if !self.folders.contains(&folder) {
            self.folders.push(folder);
        }
    }

    pub fn remove_folder(&mut self, folder: &Path) {
        self.folders.retain(|f| f != folder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_folder_deduplicates() {
        let mut config = Config::default();
        config.add_folder(PathBuf::from("/home/user/Music"));
        config.add_folder(PathBuf::from("/home/user/Music"));
        assert_eq!(config.folders, vec![PathBuf::from("/home/user/Music")]);
    }

    #[test]
    fn remove_folder_removes_only_matching_entry() {
        let mut config = Config::default();
        config.add_folder(PathBuf::from("/a"));
        config.add_folder(PathBuf::from("/b"));
        config.remove_folder(Path::new("/a"));
        assert_eq!(config.folders, vec![PathBuf::from("/b")]);
    }

    #[test]
    fn round_trips_through_json() {
        let mut config = Config::default();
        config.add_folder(PathBuf::from("/shared/photos"));
        let json = serde_json::to_string(&config).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.folders, restored.folders);
    }
}
