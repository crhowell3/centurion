use std::{env, mem};
mod screen;

use crate::screen::dashboard;
use crate::screen::Screen;

use iced::{Element, Subscription, Task, Theme};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    args.next();

    let version = args.next().is_some_and(|s| s == "--version" || s == "-v");

    if version {
        println!("centurion {}", "0.1.0");

        return Ok(());
    }

    iced::daemon(move || Centurion::new(), Centurion::update, Centurion::view)
        .title(Centurion::title)
        .theme(Centurion::theme)
        .subscription(Centurion::subscription)
        .settings(settings)
        .run()
        .inspect_err(|err| log::error!("{}", err))?;

    Ok(())
}

fn settings(config_load: &Result<Config, config::Error>) -> iced::Settings {
    let default_text_size = config_load
        .as_ref()
        .ok()
        .and_then(|config| config.font.size)
        .map_or(theme::TEXT_SIZE, f32::from);
}

struct Centurion {
    screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    Dashboard(dashboard::Message),
}

impl Centurion {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Dashboard(dashboard::Dashboard::new(true)),
            },
            Task::none(),
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        String::from("Centurion")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dashboard(message) => {
                let Screen::Dashboard(dashboard) = &mut self.screen else {
                    return Task::none();
                };

                match dashboard.update(message) {
                    Some(dashboard::Event::RefreshConfiguration) => Task::none(),
                    None => Task::none(),
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Dashboard(dashboard) => dashboard.view().map(Message::Dashboard),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let screen = match &self.screen {
            Screen::Dashboard(_dashboard) => Subscription::none(),
        };

        Subscription::batch([screen])
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
