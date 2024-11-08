#![allow(clippy::large_enum_variant, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appearance;
mod event;
mod screen;
mod widget;
mod window;

use std::time::{Duration, Instant};
use std::{env, mem};

use appearance::{theme, Theme};
use data::environment;
use iced::widget::{column, container};
use iced::{padding, Length, Subscription, Task};
use screen::{dashboard, help, welcome};
use tokio::runtime;

use self::widget::Element;
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

    let is_debug = cfg!(debug_assertions);

    let config_load = {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        rt.block_on(Config::load())
    };

    iced::daemon("Centurion", Centurion::update, Centurion::view)
        .theme(Centurion::theme)
        .scale_factor(Centurion::scale_factor)
        .subscription(Centurion::subscription)
        .run_with(move || Centurion::new(config_load))
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
            }) => (Config::default(), Task::none()),
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
}

#[derive(Debug)]
pub enum Message {
    AppearanceReloaded(data::appearance::Appearance),
    ScreenConfigReloaded(Result<Config, config::Error>),
    Dashboard(dashboard::Message),
    Help(help::Message),
    Welcome(welcome::Message),
    Event(window::Id, Event),
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
        (centurion, command)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppearanceReloaded(appearance) => {
                self.config.appearance = appearance;
                Task::none()
            }
            Message::ScreenConfigReloaded(updated) => {
                let (halloy, command) = Centurion::load_from_state(self.main_window.id, updated);
                *self = halloy;
                command
            }
            Message::Dashboard(message) => {
                let Screen::Dashboard(dashboard) = &mut self.screen else {
                    return Task::none();
                };

                let (command, event) = dashboard.update(
                    message,
                    &mut self.theme,
                    &self.version,
                    &self.config,
                    &self.main_window,
                );

                let event_task = match event {
                    Some(dashboard::Event::ConfigReloaded(config)) => {
                        match config {
                            Ok(updated) => {
                                self.theme = appearance::theme(&updated.appearance.selected).into();
                                self.config = updated;
                            }
                        };
                        Task::none()
                    }
                    Some(dashboard::Event::ReloadThemes) => Task::future(Config::load())
                        .and_then(|config| Task::done(config.appearance))
                        .map(Message::AppearanceReloaded),
                    Some(dashboard::Event::Exit) => iced::exit(),
                    None => Task::none(),
                };

                Task::batch(vec![
                    event_task,
                    command.map(Message::Dashboard),
                    track.map(Message::Dashboard),
                ])
            }
            Message::Version(remote) => {
                // Set latest known remote version
                self.version.remote = remote;

                Task::none()
            }
            Message::Help(message) => {
                let Screen::Help(help) = &mut self.screen else {
                    return Task::none();
                };

                match help.update(message) {
                    Some(help::Event::RefreshConfiguration) => {
                        Task::perform(Config::load(), Message::ScreenConfigReloaded)
                    }
                    None => Task::none(),
                }
            }
            Message::Welcome(message) => {
                let Screen::Welcome(welcome) = &mut self.screen else {
                    return Task::none();
                };

                match welcome.update(message) {
                    Some(welcome::Event::RefreshConfiguration) => {
                        Task::perform(Config::load(), Message::ScreenConfigReloaded)
                    }
                    None => Task::none(),
                }
            }
            Message::Event(window, event) => {
                if let Screen::Dashboard(dashboard) = &mut self.screen {
                    return dashboard
                        .handle_event(
                            window,
                            event,
                            &self.version,
                            &self.config,
                            &mut self.theme,
                            &self.main_window,
                        )
                        .map(Message::Dashboard);
                }

                Task::none()
            }
            Message::Window(id, event) => {
                if id == self.main_window.id {
                    match event {
                        window::Event::Moved(position) => {
                            self.main_window.position = Some(position)
                        }
                        window::Event::Resized(size) => self.main_window.size = size,
                        window::Event::Focused => self.main_window.focused = true,
                        window::Event::Unfocused => self.main_window.focused = false,
                        window::Event::Opened { position, size } => {
                            self.main_window.opened(position, size)
                        }
                        window::Event::CloseRequested => {
                            if let Screen::Dashboard(dashboard) = &mut self.screen {
                                return dashboard.exit().map(Message::Dashboard);
                            } else {
                                return iced::exit();
                            }
                        }
                    }

                    Task::perform(
                        data::Window::from(self.main_window).save(),
                        Message::WindowSettingsSaved,
                    )
                } else if let Screen::Dashboard(dashboard) = &mut self.screen {
                    dashboard
                        .handle_window_event(id, event, &mut self.theme)
                        .map(Message::Dashboard)
                } else {
                    Task::none()
                }
            }
            Message::WindowSettingsSaved(result) => {
                if let Err(err) = result {
                    log::error!("window settings failed to save: {:?}", err)
                }

                Task::none()
            }
            Message::AppearanceChange(mode) => {
                if let data::appearance::Selected::Dynamic { light, dark } =
                    &self.config.appearance.selected
                {
                    self.theme = match mode {
                        appearance::Mode::Dark => dark.clone().into(),
                        appearance::Mode::Light => light.clone().into(),
                    }
                }

                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        let content = if window_id == self.main_window.id {
            let screen = match &self.screen {
                Screen::Dashboard(dashboard) => dashboard
                    .view(&self.version, &self.config, &self.theme, &self.main_window)
                    .map(Message::Dashboard),
                Screen::Help(help) => help.view().map(Message::Help),
                Screen::Welcome(welcome) => welcome.view().map(Message::Welcome),
            };

            let content = container(screen)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::container::general);
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
        subscription::from_recipe(Message::AppearanceChange)
    }
}
