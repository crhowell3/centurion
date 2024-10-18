use std::{env, mem};

use iced::widget::{
    button, center, column, container, horizontal_space, scrollable, text, text_input,
};
use iced::window;
use iced::{Center, Element, Fill, Subscription, Task, Theme, Vector};

use self::window::Window;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    args.next();

    let version = args
        .next()
        .map(|s| s == "--version" || s == "-v")
        .unwrap_or_default();

    if version {
        println!("centurion {}", env!("CARGO_PKG_VERSION"));

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

struct Centurion {
    version: Version,
    screen: Screen,
    theme: Theme,
    config: Config,
    main_window: Window,
}

#[derive(Debug, Clone)]
enum Message {
    AppearanceReload(),
    ScreenConfigReload(Result<Config, config::Error>),
    Dashboard(dashboard::Message),
    Help(help::Message),
    Event(window::Id, Event),
    Tick(Instant),
}

impl Centurion {
    fn new() -> (Self, Task<Message>) {
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
        if let Some(window) = self.windows.get(&window_id) {
            center(window.view(window_id)).into()
        } else {
            horizontal_space().into()
        }
    }

    fn theme(&self, window: window::Id) -> Theme {
        if let Some(window) = self.windows.get(&window) {
            window.theme.clone()
        } else {
            Theme::default()
        }
    }

    fn scale_factor(&self, window: window::Id) -> f64 {
        self.windows
            .get(&window)
            .map(|window| window.current_scale)
            .unwrap_or(1.0)
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
