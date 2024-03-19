use crate::window::SpecialDirsList;

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

use serde::{Deserialize, Serialize};
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub special_dirs: SpecialDirsList,
    pub file_manager: Box<str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            special_dirs: vec![],
            file_manager: Box::from("cosmic-files"),
        }
    }
}
