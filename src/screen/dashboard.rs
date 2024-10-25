use chrono::{DateTime, Utc};
use data::dashboard::BufferAction;
use data::environment::{RELEASE_WEBSITE, WIKI_WEBSITE};
use data::history::ReadMarker;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{convert, slice};

use data::config;
use data::file_transfer;
use data::history::manager::Broadcast;
use data::user::Nick;
use data::{client, environment, history, Config, Server, Version};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{column, container, row, Space};
use iced::{clipboard, Length, Task, Vector};

use self::command_bar::CommandBar;
use self::pane::Pane;
use self::sidebar::Sidebar;
use self::theme_editor::ThemeEditor;
use crate::buffer::{self, Buffer};
use crate::widget::{
    anchored_overlay, context_menu, selectable_text, shortcut, Column, Element, Row,
};
use crate::window::Window;
use crate::{event, notification, theme, window, Theme};

mod command_bar;
pub mod pane;
pub mod sidebar;
mod theme_editor;

const SAVE_AFTER: Duration = Duration::from_secs(3);

pub struct Dashboard {
    panes: Panes,
    focus: Option<(window::Id, pane_grid::Pane)>,
    side_menu: Sidebar,
    history: history::Manager,
    last_changed: Option<Instant>,
    command_bar: Option<CommandBar>,
    file_transfers: file_transfer::Manager,
    theme_editor: Option<ThemeEditor>,
}

#[derive(Debug)]
pub enum Message {
    Pane(window::Id, pane::Message),
    Sidebar(sidebar::Message),
    SelectedText(Vec<(f32, String)>),
    History(history::manager::Message),
    DashboardSaved(Result<(), data::dashboard::Error>),
    Task(command_bar::Message),
    Shortcut(shortcut::Command),
    FileTransfer(file_transfer::task::Update),
    SendFileSelected(Server, Nick, Option<PathBuf>),
    CloseContextMenu(window::Id, bool),
    ThemeEditor(theme_editor::Message),
    ConfigReloaded(Result<Config, config::Error>),
}

#[derive(Debug)]
pub enum Event {
    ConfigReloaded(Result<Config, config::Error>),
    ReloadThemes,
    QuitServer(Server),
    Exit,
}

impl Dashboard {
    pub fn empty(config: &Config) -> (Self, Task<Message>) {
        let (main_panes, _) = pane_grid::State::new(Pane::new(Buffer::Empty, config));

        let mut dashboard = Dashboard {
            panes: Panes {
                main: main_panes,
                popout: HashMap::new(),
            },
            focus: None,
            side_menu: Sidebar::new(),
            history: history::Manager::default(),
            last_changed: None,
            command_bar: None,
            file_transfers: file_transfer::Manager::new(config.file_transfer.clone()),
            theme_editor: None,
        };

        let command = dashboard.track();

        (dashboard, command)
    }
}
