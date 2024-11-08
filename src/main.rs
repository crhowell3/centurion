use iced::system;
use iced::{Element, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Centurion::title, Centurion::update, Centurion::view)
        .font(icon::FONT_BYTES)
        .subscription(Centurion::subscription)
        .theme(Centurion::theme)
        .run_with(Centurion::new)
}

struct Centurion {
    screen: Screen,
}

#[derive(Debug, Clone)]
enum Message {
    Escape,
}

impl Centurion {
    pub fn new() -> Self {
        Self {
            screen: Screen::Loading,
        }
    }

    fn title(&self) -> String {
        "Centurion".to_owned()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Escape => Task::none(),
        }
    }
}
