use crate::window::Window;
use config::{Config, FilesConfig, CONFIG_VERSION, FILES_CONFIG_VERSION, FILES_ID};
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
use window::Flags;

mod config;
mod localize;
mod window;

fn main() -> cosmic::iced::Result {
    localize::localize();

    let (config_handler, config) = match cosmic_config::Config::new(window::ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            eprintln!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };
    let (_, files_config) = match cosmic_config::Config::new(FILES_ID, FILES_CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match FilesConfig::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            eprintln!("failed to create config handler: {}", err);
            (None, FilesConfig::default())
        }
    };
    let flags = Flags {
        files_config,
        config_handler,
        config,
    };
    cosmic::applet::run::<Window>(true, flags)
}
