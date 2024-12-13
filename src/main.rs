mod screen;

use crate::screen::dashboard;
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

    fn title(&self) -> String {
        "Centurion".to_owned()
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
