use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process;

use cosmic::app::Core;
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Command, Limits};
use cosmic::iced_core::{Alignment, Length};
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::widget::{self};
use cosmic::{cosmic_config, Element, Theme};

use crate::config::{Config, CONFIG_VERSION};

pub const ID: &str = "dev.dominiccgeh.CosmicAppletPlacesStatusIndicator";

type SpecialDirsMap = HashMap<PathBuf, &'static str>;
pub type SpecialDirsList = Vec<(PathBuf, String)>;

fn load_dirs() -> (SpecialDirsMap, SpecialDirsList) {
    let cap = 10;
    let mut special_dirs = Vec::with_capacity(cap);
    let mut special_dirs_map = HashMap::with_capacity(cap);
    for (dir_opt, dir_icon) in vec![
        // todo configure vec config
        (dirs::public_dir(), "folder-publicshare"),
        (dirs::document_dir(), "folder-documents"),
        (dirs::template_dir(), "folder-template"),
        (dirs::home_dir(), "user-home"),
        (dirs::desktop_dir(), "user-desktop"),
        (dirs::download_dir(), "folder-download"),
        (dirs::audio_dir(), "folder-music"),
        (dirs::picture_dir(), "folder-pictures"),
        (dirs::video_dir(), "folder-videos"),
    ] {
        if let Some(dir) = dir_opt {
            let Some(file_name) = dir.file_name().and_then(|x| x.to_str()) else {
                continue;
            };
            let file_name = file_name.to_owned();
            special_dirs.push((dir.clone(), file_name));
            special_dirs_map.insert(dir, dir_icon);
        }
    }
    special_dirs.push((PathBuf::from(""), "Trash".to_owned()));
    special_dirs.push((PathBuf::from("/"), "Filesystem".to_owned()));
    special_dirs_map.insert(PathBuf::from("/"), "drive-harddisk-system-symbolic");
    special_dirs.sort_by(|a, b| a.1.cmp(&b.1));
    (special_dirs_map, special_dirs)
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

pub struct Window {
    core: Core,
    popup: Option<Id>,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    special_dirs_map: SpecialDirsMap,
    special_dirs_vec: SpecialDirsList,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Open(Location),
    Config(Config),
}
#[derive(Debug, Clone)]
pub enum Location {
    Trash,
    Path(PathBuf),
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let (special_dirs_map, special_dirs_vec) = load_dirs();
        let window = Window {
            config: flags.config,
            config_handler: flags.config_handler,
            core,
            special_dirs_map,
            special_dirs_vec,
            popup: None,
        };
        (window, Command::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        // Helper for updating config values efficiently
        #[allow(unused_macros)]
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("failed to save config {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        eprintln!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name),
                        );
                    }
                }
            };
        }

        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(300.0)
                        .min_width(100.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }

            Message::Open(path) => {
                let arg = match path {
                    Location::Trash => Cow::from(OsStr::new("--trash")),
                    Location::Path(path) => Cow::from(path.into_os_string()),
                };
                _ = process::Command::new("cosmic-files").arg(arg).spawn();
            }
            Message::Config(config) => {
                if config != self.config {
                    self.config = config
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        if self.config.show_icon
            || matches!(
                self.core.applet.anchor,
                PanelAnchor::Left | PanelAnchor::Right
            )
        {
            self.core
                .applet
                .icon_button("com.system76.CosmicFiles")
                .on_press(Message::TogglePopup)
                .into()
        } else {
            let padding = self.core.applet.suggested_padding(true);
            widget::button(widget::text("Places").size(14.0))
                .on_press(Message::TogglePopup)
                .padding([padding / 2, padding])
                .style(cosmic::theme::Button::AppletIcon)
                .into()
        }
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        #[allow(unused_variables)]
        let Spacing {
            space_xs,
            space_xxs,
            space_xxxs,
            ..
        } = self.core.system_theme().cosmic().spacing;

        let mut content_list =
            widget::column::with_capacity(self.special_dirs_vec.len()).padding([8, 0]);

        for (path, name) in &self.special_dirs_vec {
            let icon = if name == "Trash" {
                widget::icon::from_name("user-trash-full-symbolic")
                    .size(16)
                    .into()
            } else {
                self.folder_icon(&path)
            };
            let open_loc = if name == "Trash" {
                Location::Trash
            } else {
                Location::Path(path.to_owned())
            };
            let row = widget::row::with_children(vec![
                icon,
                widget::text(name.clone()).width(Length::Fill).into(),
            ])
            .align_items(Alignment::Center)
            .spacing(space_xxs);
            let btn = widget::button(row)
                .on_press(Message::Open(open_loc))
                .style(cosmic::theme::Button::HeaderBar);
            // todo dynamic sizing
            let container = widget::container(btn)
                .width(Length::Fill)
                .padding([space_xxxs, space_xxs]);
            content_list = content_list.push(container);
        }

        self.core.applet.popup_container(content_list).into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        return cosmic_config::config_subscription(
            std::any::TypeId::of::<ConfigSubscription>(),
            Self::APP_ID.into(),
            CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::Config(update.config)
        });
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

impl Window {
    fn folder_icon(&self, path: &Path) -> Element<Message> {
        widget::icon::from_name(self.special_dirs_map.get(path).map_or("folder", |x| x))
            .size(16)
            .into()
    }
}
