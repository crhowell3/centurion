use data::dashboard::BufferAction;
use data::environment::{RELEASE_WEBSITE, WIKI_WEBSITE};
use data::history::ReadMarker;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use data::config;
use data::{environment, Config, Version};
use iced::Task;

use self::sidebar::Sidebar;
use self::theme_editor::ThemeEditor;
use crate::buffer::{self, Buffer};
use crate::widget::{
    anchored_overlay, context_menu, selectable_text, shortcut, Column, Element, Row,
};
use crate::{event, notification, theme, window, Theme};

pub mod sidebar;
mod theme_editor;

const SAVE_AFTER: Duration = Duration::from_secs(3);

pub struct Dashboard {
    side_menu: Sidebar,
    last_changed: Option<Instant>,
    theme_editor: Option<ThemeEditor>,
}

#[derive(Debug)]
pub enum Message {
    Sidebar(sidebar::Message),
    DashboardSaved(Result<(), data::dashboard::Error>),
    CloseContextMenu(window::Id, bool),
    ThemeEditor(theme_editor::Message),
    ConfigReloaded(Result<Config, config::Error>),
}

#[derive(Debug)]
pub enum Event {
    ConfigReloaded(Result<Config, config::Error>),
    ReloadThemes,
    Exit,
}

impl Dashboard {
    pub fn empty(config: &Config) -> (Self, Task<Message>) {
        let mut dashboard = Dashboard {
            side_menu: Sidebar::new(),
            last_changed: None,
            theme_editor: None,
        };

        let command = dashboard.track();

        (dashboard, command)
    }

    pub fn update(
        &mut self,
        message: Message,
        theme: &mut Theme,
        version: &Version,
        config: &Config,
        main_window: &Window,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::Sidebar(message) => {
                let (command, event) = self.side_menu.update(message);

                let Some(event) = event else {
                    return (command.map(Message::Sidebar), None);
                };

                let (event_task, event) = match event {
                    sidebar::Event::Open(buffer) => (
                        self.open_buffer(
                            main_window,
                            data::Buffer::Upstream(buffer),
                            config.buffer.clone().into(),
                        ),
                        None,
                    ),
                    sidebar::Event::Popout(buffer) => (
                        self.open_popout_window(
                            main_window,
                            Pane::new(Buffer::from(data::Buffer::Upstream(buffer)), config),
                        ),
                        None,
                    ),
                    sidebar::Event::Focus(window, pane) => {
                        (self.focus_pane(main_window, window, pane), None)
                    }
                    sidebar::Event::Replace(window, buffer, pane) => {
                        if let Some(state) = self.panes.get_mut(main_window.id, window, pane) {
                            state.buffer = Buffer::from(data::Buffer::Upstream(buffer));
                            self.last_changed = Some(Instant::now());
                            self.focus = None;
                            (
                                Task::batch(vec![
                                    self.reset_pane(main_window, window, pane),
                                    self.focus_pane(main_window, window, pane),
                                ]),
                                None,
                            )
                        } else {
                            (Task::none(), None)
                        }
                    }
                    sidebar::Event::Close(window, pane) => {
                        if self.focus == Some((window, pane)) {
                            self.focus = None;
                        }

                        (self.close_pane(main_window, window, pane), None)
                    }
                    sidebar::Event::Swap(from_window, from_pane, to_window, to_pane) => {
                        self.last_changed = Some(Instant::now());

                        if from_window == main_window.id && to_window == main_window.id {
                            self.panes.main.swap(from_pane, to_pane);
                            (self.focus_pane(main_window, from_window, from_pane), None)
                        } else {
                            if let Some((from_state, to_state)) = self
                                .panes
                                .get(main_window.id, from_window, from_pane)
                                .cloned()
                                .zip(self.panes.get(main_window.id, to_window, to_pane).cloned())
                            {
                                if let Some(state) =
                                    self.panes.get_mut(main_window.id, from_window, from_pane)
                                {
                                    *state = to_state;
                                }
                                if let Some(state) =
                                    self.panes.get_mut(main_window.id, to_window, to_pane)
                                {
                                    *state = from_state;
                                }
                            }
                            (Task::none(), None)
                        }
                    }
                    sidebar::Event::Leave(buffer) => self.leave_buffer(main_window, buffer),
                    sidebar::Event::ToggleInternalBuffer(buffer) => (
                        self.toggle_internal_buffer(config, main_window, buffer),
                        None,
                    ),
                    sidebar::Event::ToggleCommandBar => (
                        self.toggle_command_bar(
                            &closed_buffers(self, main_window.id),
                            version,
                            config,
                            theme,
                            main_window,
                        ),
                        None,
                    ),
                    sidebar::Event::ConfigReloaded(conf) => {
                        (Task::none(), Some(Event::ConfigReloaded(conf)))
                    }
                    sidebar::Event::OpenReleaseWebsite => {
                        let _ = open::that_detached(RELEASE_WEBSITE);
                        (Task::none(), None)
                    }
                    sidebar::Event::ToggleThemeEditor => {
                        (self.toggle_theme_editor(theme, main_window), None)
                    }
                    sidebar::Event::OpenDocumentation => {
                        let _ = open::that_detached(WIKI_WEBSITE);
                        (Task::none(), None)
                    }
                };

                return (
                    Task::batch(vec![event_task, command.map(Message::Sidebar)]),
                    event,
                );
            }
            Message::DashboardSaved(Ok(_)) => {
                log::info!("dashboard saved");
            }
            Message::DashboardSaved(Err(error)) => {
                log::warn!("error saving dashboard: {error}");
            }
            Message::ThemeEditor(message) => {
                let mut editor_event = None;
                let mut event = None;
                let mut tasks = vec![];

                if let Some(editor) = self.theme_editor.as_mut() {
                    let (task, event) = editor.update(message, theme);

                    tasks.push(task.map(Message::ThemeEditor));
                    editor_event = event;
                }

                if let Some(editor_event) = editor_event {
                    match editor_event {
                        theme_editor::Event::Close => {
                            if let Some(editor) = self.theme_editor.take() {
                                tasks.push(window::close(editor.window));
                            }
                        }
                        theme_editor::Event::ReloadThemes => {
                            event = Some(Event::ReloadThemes);
                        }
                    }
                }

                return (Task::batch(tasks), event);
            }
            Message::ConfigReloaded(config) => {
                return (Task::none(), Some(Event::ConfigReloaded(config)));
            }
        }

        (Task::none(), None)
    }

    pub fn view_window<'a>(
        &'a self,
        window: window::Id,
        config: &'a Config,
        theme: &'a Theme,
        main_window: &'a Window,
    ) -> Element<'a, Message> {
        if let Some(state) = self.panes.popout.get(&window) {
            let content = container(
                PaneGrid::new(state, |id, pane, _maximized| {
                    let is_focused = self.focus == Some((window, id));
                    pane.view(
                        id,
                        window,
                        1,
                        is_focused,
                        false,
                        &self.file_transfers,
                        &self.history,
                        &self.side_menu,
                        config,
                        theme,
                        main_window,
                    )
                })
                .spacing(4)
                .on_click(pane::Message::PaneClicked),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8);

            return Element::new(content).map(move |message| Message::Pane(window, message));
        } else if let Some(editor) = self.theme_editor.as_ref() {
            if editor.window == window {
                return editor.view(theme).map(Message::ThemeEditor);
            }
        }

        column![].into()
    }

    pub fn view<'a>(
        &'a self,
        version: &'a Version,
        config: &'a Config,
        theme: &'a Theme,
        main_window: &'a Window,
    ) -> Element<'a, Message> {
        let content = match config.sidebar.position {
            data::config::sidebar::Position::Left | data::config::sidebar::Position::Top => {
                vec![side_menu.unwrap_or_else(|| row![].into()), pane_grid.into()]
            }
            data::config::sidebar::Position::Right | data::config::sidebar::Position::Bottom => {
                vec![pane_grid.into(), side_menu.unwrap_or_else(|| row![].into())]
            }
        };

        let base: Element<Message> = if config.sidebar.position.is_horizontal() {
            Column::with_children(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            Row::with_children(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let base = if let Some(command_bar) = self.command_bar.as_ref() {
            let background = anchored_overlay(
                base,
                container(Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::container::transparent_overlay),
                anchored_overlay::Anchor::BelowTopCentered,
                0.0,
            );

            // Task bar
            anchored_overlay(background, anchored_overlay::Anchor::BelowTopCentered, 10.0)
        } else {
            // Align `base` into same view tree shape
            // as `anchored_overlay` to prevent diff
            // from firing when displaying command bar
            column![column![base]].into()
        };

        shortcut(base, config.keyboard.shortcuts(), Message::Shortcut)
    }

    pub fn handle_event(
        &mut self,
        window: window::Id,
        event: event::Event,
        version: &Version,
        config: &Config,
        theme: &mut Theme,
        main_window: &Window,
    ) -> Task<Message> {
        use event::Event::*;

        match event {
            Escape => {
                // Order of operations
                //
                // - Close command bar (if main window)
                // - Close context menu
                // - Restore maximized pane (if main window)
                // - Unfocus
                if self.command_bar.is_some() && window == main_window.id {
                    self.toggle_command_bar(
                        &closed_buffers(self, main_window.id),
                        version,
                        config,
                        theme,
                        main_window,
                    )
                } else {
                    context_menu::close(convert::identity)
                        .map(move |any_closed| Message::CloseContextMenu(window, any_closed))
                }
            }
            Copy => selectable_text::selected(Message::SelectedText),
            Home => self
                .get_focused_mut(main_window)
                .map(|(window, id, pane)| {
                    pane.buffer.scroll_to_start().map(move |message| {
                        Message::Pane(window, pane::Message::Buffer(id, message))
                    })
                })
                .unwrap_or_else(Task::none),
            End => self
                .get_focused_mut(main_window)
                .map(|(window, pane, state)| {
                    state.buffer.scroll_to_end().map(move |message| {
                        Message::Pane(window, pane::Message::Buffer(pane, message))
                    })
                })
                .unwrap_or_else(Task::none),
        }
    }

    fn toggle_theme_editor(&mut self, theme: &mut Theme, main_window: &Window) -> Task<Message> {
        if let Some(editor) = self.theme_editor.take() {
            *theme = theme.selected();
            window::close(editor.window)
        } else {
            let (editor, task) = ThemeEditor::open(main_window);

            self.theme_editor = Some(editor);

            task.then(|_| Task::none())
        }
    }

    fn from_data(
        data: data::Dashboard,
        config: &Config,
        main_window: &Window,
    ) -> (Self, Task<Message>) {
        use pane_grid::Configuration;

        fn configuration(pane: data::Pane) -> Configuration<Pane> {
            match pane {
                data::Pane::Split { axis, ratio, a, b } => Configuration::Split {
                    axis: match axis {
                        data::pane::Axis::Horizontal => pane_grid::Axis::Horizontal,
                        data::pane::Axis::Vertical => pane_grid::Axis::Vertical,
                    },
                    ratio,
                    a: Box::new(configuration(*a)),
                    b: Box::new(configuration(*b)),
                },
                data::Pane::Buffer { buffer, settings } => {
                    Configuration::Pane(Pane::with_settings(Buffer::from(buffer), settings))
                }
                data::Pane::Empty => Configuration::Pane(Pane::with_settings(
                    Buffer::empty(),
                    buffer::Settings::default(),
                )),
            }
        }

        let mut dashboard = Self {
            panes: Panes {
                main: pane_grid::State::with_configuration(configuration(data.pane)),
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

        let mut tasks = vec![];

        for pane in data.popout_panes {
            // Popouts are only a single pane
            let Configuration::Pane(pane) = configuration(pane) else {
                continue;
            };

            tasks.push(dashboard.open_popout_window(main_window, pane));
        }

        (dashboard, Task::batch(tasks))
    }

    pub fn handle_window_event(
        &mut self,
        id: window::Id,
        event: window::Event,
        theme: &mut Theme,
    ) -> Task<Message> {
        if self.panes.popout.contains_key(&id) {
            match event {
                window::Event::CloseRequested => {
                    self.panes.popout.remove(&id);
                    return window::close(id);
                }
                window::Event::Moved(_)
                | window::Event::Resized(_)
                | window::Event::Focused
                | window::Event::Unfocused
                | window::Event::Opened { .. } => {}
            }
        } else if self
            .theme_editor
            .as_ref()
            .map(|e| e.window == id)
            .unwrap_or_default()
        {
            match event {
                window::Event::CloseRequested => {
                    if let Some(editor) = self.theme_editor.take() {
                        *theme = theme.selected();
                        return window::close(editor.window);
                    }
                }
                window::Event::Moved(_)
                | window::Event::Resized(_)
                | window::Event::Focused
                | window::Event::Unfocused
                | window::Event::Opened { .. } => {}
            }
        }

        Task::none()
    }

    pub fn preview_theme_in_editor(
        &mut self,
        colors: theme::Colors,
        main_window: &Window,
        theme: &mut Theme,
    ) -> Task<Message> {
        *theme = theme.preview(data::Theme::new("Custom Theme".into(), colors));

        if let Some(editor) = &self.theme_editor {
            window::gain_focus(editor.window)
        } else {
            let (editor, task) = ThemeEditor::open(main_window);

            self.theme_editor = Some(editor);

            task.then(|_| Task::none())
        }
    }
}

fn cycle_next_buffer(
    current: Option<&buffer::Upstream>,
    mut all: Vec<buffer::Upstream>,
    opened: &[buffer::Upstream],
) -> Option<buffer::Upstream> {
    all.retain(|buffer| Some(buffer) == current || !opened.contains(buffer));

    let next = || {
        let buffer = current?;
        let index = all.iter().position(|b| b == buffer)?;
        all.get(index + 1)
    };

    next().or_else(|| all.first()).cloned()
}

fn cycle_previous_buffer(
    current: Option<&buffer::Upstream>,
    mut all: Vec<buffer::Upstream>,
    opened: &[buffer::Upstream],
) -> Option<buffer::Upstream> {
    all.retain(|buffer| Some(buffer) == current || !opened.contains(buffer));

    let previous = || {
        let buffer = current?;
        let index = all.iter().position(|b| b == buffer).filter(|i| *i > 0)?;

        all.get(index - 1)
    };

    previous().or_else(|| all.last()).cloned()
}
