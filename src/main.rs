mod screen;

use crate::screen::help;
use crate::screen::Screen;

use iced::{Element, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Centurion::title, Centurion::update, Centurion::view)
        .subscription(Centurion::subscription)
        .theme(Centurion::theme)
        .run_with(move || Centurion::new())
}

struct Centurion {
    screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    Help(help::Message),
}

impl Centurion {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Help(help::Help::new(true)),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        "Centurion".to_owned()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Help(_message) => {
                let Screen::Help(_help) = &mut self.screen else {
                    return Task::none();
                };

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Help(help) => help.view().map(Message::Help),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let screen = match &self.screen {
            Screen::Help(_help) => Subscription::none(),
        };

        Subscription::batch([screen])
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
