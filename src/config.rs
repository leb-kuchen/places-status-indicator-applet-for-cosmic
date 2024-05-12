use std::path::PathBuf;

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

use serde::{Deserialize, Serialize};
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub show_icon: bool,
}

// copied from cosmic files config.rs

impl Favorite {
    pub fn from_path(path: PathBuf) -> Self {
        // Ensure that special folders are handled properly
        for favorite in &[
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Music,
            Self::Pictures,
            Self::Videos,
        ] {
            if let Some(favorite_path) = favorite.path_opt() {
                if &favorite_path == &path {
                    return favorite.clone();
                }
            }
        }
        Self::Path(path)
    }

    pub fn path_opt(&self) -> Option<PathBuf> {
        match self {
            Self::Home => dirs::home_dir(),
            Self::Documents => dirs::document_dir(),
            Self::Downloads => dirs::download_dir(),
            Self::Music => dirs::audio_dir(),
            Self::Pictures => dirs::picture_dir(),
            Self::Videos => dirs::video_dir(),
            Self::Path(path) => Some(path.clone()),
        }
    }
}

pub const FILES_CONFIG_VERSION: u64 = 1;
pub const FILES_ID: &str = "com.system76.CosmicFiles";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Favorite {
    Home,
    Documents,
    Downloads,
    Music,
    Pictures,
    Videos,
    Path(PathBuf),
}
#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct FilesConfig {
    pub favorites: Vec<Favorite>,
}

impl Default for Config {
    fn default() -> Self {
        Self { show_icon: true }
    }
}
