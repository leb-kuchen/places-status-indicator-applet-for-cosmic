use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

use cosmic::app::Core;
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Limits, Task};
#[allow(unused_imports)]
use cosmic::iced_core::{Alignment, Length};
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::style::iced::application;
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::segmented_button::Entity;
use cosmic::widget::{self, segmented_button};
use cosmic::{cosmic_config, Element, Theme};
use once_cell::sync::Lazy;

use crate::config::{
    Config, Favorite, FilesConfig, CONFIG_VERSION, FILES_CONFIG_VERSION, FILES_ID,
};

pub const ID: &str = "dev.dominiccgeh.CosmicAppletPlacesStatusIndicator";

static SPECIAL_DIRS: Lazy<HashMap<PathBuf, &'static str>> = Lazy::new(|| {
    let mut special_dirs = HashMap::new();
    if let Some(dir) = dirs::document_dir() {
        special_dirs.insert(dir, "folder-documents");
    }
    if let Some(dir) = dirs::download_dir() {
        special_dirs.insert(dir, "folder-download");
    }
    if let Some(dir) = dirs::audio_dir() {
        special_dirs.insert(dir, "folder-music");
    }
    if let Some(dir) = dirs::picture_dir() {
        special_dirs.insert(dir, "folder-pictures");
    }
    if let Some(dir) = dirs::public_dir() {
        special_dirs.insert(dir, "folder-publicshare");
    }
    if let Some(dir) = dirs::template_dir() {
        special_dirs.insert(dir, "folder-templates");
    }
    if let Some(dir) = dirs::video_dir() {
        special_dirs.insert(dir, "folder-videos");
    }
    if let Some(dir) = dirs::desktop_dir() {
        special_dirs.insert(dir, "user-desktop");
    }
    if let Some(dir) = dirs::home_dir() {
        special_dirs.insert(dir, "user-home");
    }
    special_dirs
});

pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub files_config: FilesConfig,
}

pub struct Window {
    core: Core,
    popup: Option<Id>,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    files_config: FilesConfig,
    nav_model: segmented_button::SingleSelectModel,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    FilesConfig(FilesConfig),
    Config(Config),
    NavModelSelected(segmented_button::Entity),
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

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<cosmic::app::Message<Self::Message>>) {
        let mut window = Window {
            config: flags.config,
            config_handler: flags.config_handler,
            core,
            files_config: flags.files_config,
            popup: None,
            nav_model: segmented_button::ModelBuilder::default().build(),
        };
        window.update_nav_model();
        (window, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::app::Message<Self::Message>> {
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
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(200.0)
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

            Message::Config(config) => {
                if config != self.config {
                    self.config = config
                }
            }
            Message::FilesConfig(config) => {
                if config != self.files_config {
                    self.files_config = config;
                    self.update_nav_model()
                }
            }
            Message::NavModelSelected(id) => {
                if let Some(path) = self.nav_model.data::<Location>(id) {
                    let arg = match path {
                        Location::Trash => Cow::from(OsStr::new("--trash")),
                        Location::Path(path) => Cow::from(path.clone().into_os_string()),
                    };
                    _ = process::Command::new("cosmic-files").arg(arg).spawn();
                }
            } // Message::RemoveFavorite(entity) => {
              //     if let Some(FavoriteIndex(favorite_i)) =
              //         self.nav_model.data::<FavoriteIndex>(entity)
              //     {
              //         let mut favorites = self.files_config.favorites.clone();
              //         favorites.remove(*favorite_i);
              //         // config_set!(favorites, favorites);
              //         self.files_config.favorites = favorites;
              //         self.update_nav_model();
              //     }
              // }
        }
        Task::none()
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
            widget::button::custom(widget::text("Places").size(14.0))
                .on_press(Message::TogglePopup)
                .padding([padding / 2, padding])
                .class(cosmic::theme::Button::AppletIcon)
                .into()
        }
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        #[allow(unused_variables)]
        let Spacing {
            space_s,
            space_xs,
            space_xxs,
            space_xxxs,
            ..
        } = self.core.system_theme().cosmic().spacing;

        let mut content_list = widget::column().padding([8, 0]);

        let nav = segmented_button::vertical(&self.nav_model)
            .on_activate(Message::NavModelSelected)
            .button_height(32)
            .width(300.into())
            .button_padding([space_s, space_xxs, space_s, space_xxs])
            .button_spacing(space_xxs)
            .spacing(space_xxs)
            .style(cosmic::theme::SegmentedButton::TabBar);

        content_list = content_list.push(nav);

        self.core.applet.popup_container(content_list).into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct FilesConfigSubscription;

        let config_sub = cosmic_config::config_subscription(
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
        let files_config_sub = cosmic_config::config_subscription(
            std::any::TypeId::of::<FilesConfigSubscription>(),
            FILES_ID.into(),
            FILES_CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::FilesConfig(update.config)
        });
        Subscription::batch([config_sub, files_config_sub])
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
struct FavoriteIndex(usize);
impl Window {
    fn update_nav_model(&mut self) {
        let mut nav_model = segmented_button::ModelBuilder::default();
        for (favorite_i, favorite) in self.files_config.favorites.iter().enumerate() {
            if let Some(path) = favorite.path_opt() {
                let name = if matches!(favorite, Favorite::Home) {
                    "Home".to_string()
                } else if let Some(file_name) = path.file_name().and_then(|x| x.to_str()) {
                    file_name.to_string()
                } else {
                    continue;
                };
                nav_model = nav_model.insert(move |b| {
                    b.text(name.clone())
                        .icon(
                            widget::icon::icon(if path.is_dir() {
                                folder_icon_symbolic(&path, 16)
                            } else {
                                widget::icon::from_name("text-x-generic-symbolic")
                                    .size(16)
                                    .handle()
                            })
                            .size(16),
                        )
                        .data(Location::Path(path.clone()))
                        .data(FavoriteIndex(favorite_i))
                });
            }
        }
        nav_model = nav_model.insert(|b| {
            b.text("Trash".to_string())
                .icon(widget::icon::icon(trash_icon_symbolic(16)))
                .data(Location::Trash)
        });
        self.nav_model = nav_model.build();
    }
}

pub fn folder_icon_symbolic(path: &PathBuf, icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(format!(
        "{}-symbolic",
        SPECIAL_DIRS.get(path).map_or("folder", |x| *x)
    ))
    .size(icon_size)
    .handle()
}
pub fn trash_icon_symbolic(icon_size: u16) -> widget::icon::Handle {
    let full = match trash::os_limited::list() {
        Ok(entries) => !entries.is_empty(),
        Err(_err) => false,
    };
    widget::icon::from_name(if full {
        "user-trash-full-symbolic"
    } else {
        "user-trash-symbolic"
    })
    .size(icon_size)
    .handle()
}
