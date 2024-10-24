use crate::widget::Element;
use data::config;

pub mod reload_configuration_error;

#[derive(Debug)]
pub enum Modal {
    ReloadConfigurationError(config::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Cancel,
}

pub enum Event {
    CloseModal,
}

impl Modal {
    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Cancel => Some(Event::CloseModal),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Modal::ReloadConfigurationError(error) => reload_configuration_error::view(error),
        }
    }
}
