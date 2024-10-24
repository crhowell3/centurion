use data::environment::WIKI_WEBSITE;
use data::Config;
use iced::widget::{button, column, container, image, row, text, vertical_space};
use iced::{alignment, Length};

use crate::theme;
use crate::widget::Element;

#[derive(Debug, Clone)]
pub enum Message {
    RefreshConfiguration,
    OpenConfigurationDirectory,
    OpenWikiWebsite,
}

#[derive(Debug, Clone)]
pub enum Event {
    RefreshConfiguration,
}

#[derive(Debug, Default, Clone)]
pub struct Welcome;

impl Welcome {
    pub fn new() -> Self {
        Config::create_initial_config();

        Welcome
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::RefreshConfiguration => Some(Event::RefreshConfiguration),
            Message::OpenConfigurationDirectory => {
                let _ = open::that_detached(Config::config_dir());

                None
            }
            Message::OpenWikiWebsite => {
                let _ = open::that_detached(WIKI_WEBSITE);

                None
            }
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        let config_dir = String::from(Config::config_dir().to_string_lossy());

        let config_button = button(
            container(text(config_dir))
                .align_x(alignment::Horizontal::Center)
                .width(Length::Shrink),
        )
        .padding([5, 20])
        .width(Length::Shrink)
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(Message::OpenConfigurationDirectory);
    }
}
