use iced::widget::{button, column, container, text, vertical_space};
use iced::{alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    OpenWikiWebsite,
}

#[derive(Debug, Clone)]
pub enum Event {
    RefreshConfiguration,
}

#[derive(Debug, Clone)]
pub struct Dashboard {
    test: bool,
}

impl Dashboard {
    pub fn new(test: bool) -> Self {
        Dashboard { test }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::OpenWikiWebsite => {
                let _ = open::that_detached("https://github.com/crhowell3/centurion");

                None
            }
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        let start_button = button(
            container(text("Start"))
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .padding(5)
        .width(Length::Fill)
        .on_press(Message::OpenWikiWebsite);

        let stop_button = button(
            container(text("Stop"))
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .padding(5)
        .width(Length::Fill)
        .on_press(Message::OpenWikiWebsite);

        let content = column![]
            .spacing(1)
            .push(text("Centurion"))
            .push(vertical_space().height(4))
            .push(column![].width(250).spacing(4).push(start_button))
            .push(column![].width(250).spacing(4).push(stop_button))
            .align_x(iced::Alignment::Center);

        container(content)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
