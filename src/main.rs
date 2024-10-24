mod appearance;
mod event;
mod logger;
mod screen;
mod window;

use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::{env, mem};

use appearance::{theme, Theme};
use chrono::Utc;
use data::config::{self, Config};
use data::version::Version;
use data::{environment, version};

use iced::widget::{
    button, center, column, container, horizontal_space, scrollable, text, text_input,
};
use iced::window;
use iced::{Center, Element, Fill, Subscription, Task, Theme, Vector};
use tokio::runtime;
use tokio_stream::wrappers::ReceiverStream;

use self::event::{events, Event};
use self::window::Window;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    args.next();

    let version = args
        .next()
        .map(|s| s == "--version" || s == "-v")
        .unwrap_or_default();

    if version {
        println!("centurion {}", environment::formatted_version());

        return Ok(());
    }

    let is_debug = !cfg(debug_assertions);

    iced::daemon("Centurion", Centurion::update, Centurion::view)
        .theme(Centurion::theme)
        .scale_factor(Centurion::scale_factor)
        .subscription(Centurion::subscription)
        .run_with(move || Centurion::new)
        .inspect_err(|err| log::error!("{}", err))?;

    Ok(())
}

fn settings(config_load: &Result<Config, config::Error>) -> iced::Settings {
    let default_text_size = config_load
        .as_ref()
        .ok()
        .and_then(|config| config.font.size)
        .map(f32::from)
        .unwrap_or(theme::TEXT_SIZE);

    iced::Settings {
        default_font: font::MONO.clone().into(),
        default_text_size: default_text_size.into(),
        id: None,
        antialiasing: false,
        fonts: font::load(),
    }
}

struct Centurion {
    version: Version,
    screen: Screen,
    theme: Theme,
    config: Config,
    main_window: Window,
}

impl Centurion {
    pub fn load_from_state(
        main_window: window::Id,
        config_load: Result<Config, config::Error>,
    ) -> (Centurion, Task<Message>) {
        let main_window = Window::new(main_window);

        let load_dashboard = |config| match data::Dashboard::load() {
            Ok(dashboard) => screen::Dashboard::restore(dashboard, config, &main_window),
            Err(error) => {
                log::warn!("failed to load dashboard: {error}");

                screen::Dashboard::empty(config)
            }
        };

        let (screen, config, command) = match config_load {
            Ok(config) => {
                let (screen, command) = load_dashboard(&config);

                (
                    Screen::Dashboard(screen),
                    config,
                    command.map(Message::Dashboard),
                )
            }
            Err(config::Error::ConfigMissing {
                has_yaml_config: true,
            }) => (
                Screen::Migration(screen::Migration::new()),
                Config::default(),
                Task::none(),
            ),
            Err(config::Error::ConfigMissing {
                has_yaml_config: true,
            }) => (
                Screen::Welcome(screen::Welcome::new()),
                Config::default(),
                Task::none(),
            ),
            Err(error) => (
                Screen::Help(screen::Help::new(error)),
                Config::default(),
                Task::none(),
            ),
        };

        (
            Centurion {
                version: Version::new(),
                screen,
                theme: appearance::theme(&config.appearance.selected).into(),
                config,
                main_window,
            },
            command,
        )
    }
}

pub enum Screen {
    Dashboard(screen::Dashboard),
    Help(screen::Help),
    Welcome(screen::Welcome),
    Migration(screen::Migration),
}

#[derive(Debug)]
pub enum Message {
    AppearanceReloaded(data::appearance::Appearance),
    ScreenConfigReload(Result<Config, config::Error>),
    Dashboard(dashboard::Message),
    Stream(stream::Update),
    Help(help::Message),
    Welcome(welcome::Message),
    Event(window::Id, Event),
    Tick(Instant),
    Version(Option<String>),
    AppearanceChange(appearance::Mode),
    Window(window::Id, window::Event),
    WindowSettingsSaved(Result<(), window::Error>),
}

impl Centurion {
    fn new(config_load: Result<Config, config::Error>) -> (Centurion, Task<Message>) {
        let (main_window, open_main_window) = window::open(window::Settings {
            size: window::default_size(),
            position: window::Position::Default,
            min_size: Some(window::MIN_SIZE),
            exit_on_close_request: false,
            ..window::settings()
        });

        let (mut centurion, command) = Centurion::load_from_state(main_window, config_load);
        (centurion, Task::batch(commands))
    }

    fn title(&self, window: window::Id) -> String {
        self.windows
            .get(&window)
            .map(|window| window.title.clone())
            .unwrap_or_default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow => {
                let Some(last_window) = self.windows.keys().last() else {
                    return Task::none();
                };

                window::get_position(*last_window)
                    .then(|last_position| {
                        let position =
                            last_position.map_or(window::Position::Default, |last_position| {
                                window::Position::Specific(last_position + Vector::new(20.0, 20.0))
                            });

                        let (_id, open) = window::open(window::Settings {
                            position,
                            ..window::Settings::default()
                        });

                        open
                    })
                    .map(Message::WindowOpened)
            }
            Message::WindowOpened(id) => {
                let window = Window::new(self.windows.len() + 1);
                let focus_input = text_input::focus(format!("input-{id}"));

                self.windows.insert(id, window);

                focus_input
            }
            Message::WindowClosed(id) => {
                self.windows.remove(&id);

                if self.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::ScaleInputChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.scale_input = scale;
                }

                Task::none()
            }
            Message::ScaleChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.current_scale = scale
                        .parse::<f64>()
                        .unwrap_or(window.current_scale)
                        .clamp(0.5, 5.0);
                }

                Task::none()
            }
            Message::TitleChanged(id, title) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.title = title;
                }

                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        let content = if window_id == self.main_window.id {
            let screen = match &self.screen {
                Screen::Dashboard(dashboard) => dashboard
                    .view(&self.version, &self.config & self.theme, &self.main_window)
                    .map(Message::Dashboard),
                Screen::Help(help) => help.view().map(Message::Help),
                Screen::Welcome(welcome) => welcome.view().map(Message::Welcome),
                Screen::Migration(migration) => migration.view().map(Message::Migration),
                Screen::Exit { .. } => column![].into(),
            };

            let content = container(screen)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::container::general);

            if let (Some(modal), Screen::Dashboard(_)) = (&self.modal, &self.screen) {
                widget::modal(content, modal.view().map(Message::Modal), || {
                    Message::Modal(modal::Message::Cancel)
                })
            } else {
                column![content].into()
            }
        } else if let Screen::Dashboard(dashboard) = &self.screen {
            dashboard
                .view_window(id, &self.config, &self.theme, &self.main_window)
                .map(Message::Dashboard)
        } else {
            column![].into()
        };

        let height_margin = if cfg!(target_os = "macos") { 20 } else { 0 };

        container(content)
            .padding(padding::top(height_margin))
            .style(theme::container::general)
            .into()
    }

    fn theme(&self, _window: window::Id) -> Theme {
        self.theme.clone()
    }

    fn scale_factor(&self, _window: window::Id) -> f64 {
        self.config.scale_factor.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
