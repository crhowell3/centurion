mod screen;

use crate::screen::welcome;
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
    Welcome(welcome::Message),
}

impl Centurion {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Welcome(welcome::Welcome::new(true)),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        "Centurion".to_owned()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Welcome(message) => {
                let Screen::Welcome(welcome) = &mut self.screen else {
                    return Task::none();
                };

                match welcome.update(message) {
                    Some(welcome::Event::RefreshConfiguration) => Task::none(),
                    None => Task::none(),
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Welcome(welcome) => welcome.view().map(Message::Welcome),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let screen = match &self.screen {
            Screen::Welcome(_welcome) => Subscription::none(),
        };

        Subscription::batch([screen])
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
