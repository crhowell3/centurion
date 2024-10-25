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
    SelectedText(Vec<(f32, String)>),
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

    pub fn restore(
        dashboard: data::Dashboard,
        config: &Config,
        main_window: &Window,
    ) -> (Self, Task<Message>) {
        let (mut dashboard, task) = Dashboard::from_data(dashboard, config, main_window);

        let command = if let Some((pane, _)) = dashboard.panes.main.panes.iter().next() {
            Task::batch(vec![
                dashboard.focus_pane(main_window, main_window.id, *pane),
                dashboard.track(),
            ])
        } else {
            dashboard.track()
        };

        (dashboard, Task::batch(vec![task, command]))
    }

    pub fn update(
        &mut self,
        message: Message,
        clients: &mut client::Map,
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
                    sidebar::Event::Leave(buffer) => {
                        self.leave_buffer(main_window, clients, buffer)
                    }
                    sidebar::Event::ToggleInternalBuffer(buffer) => (
                        self.toggle_internal_buffer(config, main_window, buffer),
                        None,
                    ),
                    sidebar::Event::ToggleCommandBar => (
                        self.toggle_command_bar(
                            &closed_buffers(self, main_window.id, clients),
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
            Message::SelectedText(contents) => {
                let mut last_y = None;
                let contents = contents
                    .into_iter()
                    .fold(String::new(), |acc, (y, content)| {
                        if let Some(_y) = last_y {
                            let new_line = if y == _y { "" } else { "\n" };
                            last_y = Some(y);

                            format!("{acc}{new_line}{content}")
                        } else {
                            last_y = Some(y);

                            content
                        }
                    });

                if !contents.is_empty() {
                    return (clipboard::write(contents), None);
                }
            }
            Message::History(message) => {
                if let Some(event) = self.history.update(message) {
                    match event {
                        history::manager::Event::Loaded(kind) => {
                            let buffer = kind.into();

                            if let Some((window, pane, state)) =
                                self.panes.get_mut_by_buffer(main_window.id, &buffer)
                            {
                                return (
                                    state.buffer.scroll_to_backlog(&self.history, config).map(
                                        move |message| {
                                            Message::Pane(
                                                window,
                                                pane::Message::Buffer(pane, message),
                                            )
                                        },
                                    ),
                                    None,
                                );
                            }
                        }
                        history::manager::Event::Closed(kind, read_marker) => {
                            if let Some(((server, target), read_marker)) =
                                kind.server().zip(kind.target()).zip(read_marker)
                            {
                                clients.send_markread(server, target, read_marker);
                            }
                        }
                        history::manager::Event::Exited(results) => {
                            for (kind, read_marker) in results {
                                if let Some(((server, target), read_marker)) =
                                    kind.server().zip(kind.target()).zip(read_marker)
                                {
                                    clients.send_markread(server, target, read_marker);
                                }
                            }

                            return (Task::none(), Some(Event::Exit));
                        }
                    }
                }
            }
            Message::DashboardSaved(Ok(_)) => {
                log::info!("dashboard saved");
            }
            Message::DashboardSaved(Err(error)) => {
                log::warn!("error saving dashboard: {error}");
            }
            Message::Task(message) => {
                let Some(command_bar) = &mut self.command_bar else {
                    return (Task::none(), None);
                };

                match command_bar.update(message) {
                    Some(command_bar::Event::ThemePreview(preview)) => match preview {
                        Some(preview) => *theme = theme.preview(preview),
                        None => *theme = theme.selected(),
                    },
                    Some(command_bar::Event::Command(command)) => {
                        let (command, event) = match command {
                            command_bar::Command::Version(command) => match command {
                                command_bar::Version::Application(_) => {
                                    let _ = open::that_detached(RELEASE_WEBSITE);
                                    (Task::none(), None)
                                }
                            },
                            command_bar::Command::Buffer(command) => match command {
                                command_bar::Buffer::Maximize(_) => {
                                    self.maximize_pane();
                                    (Task::none(), None)
                                }
                                command_bar::Buffer::New => (
                                    self.new_pane(pane_grid::Axis::Horizontal, config, main_window),
                                    None,
                                ),
                                command_bar::Buffer::Close => {
                                    if let Some((window, pane)) = self.focus {
                                        (self.close_pane(main_window, window, pane), None)
                                    } else {
                                        (Task::none(), None)
                                    }
                                }
                                command_bar::Buffer::Replace(buffer) => {
                                    let mut commands = vec![];

                                    if let Some((window, pane)) = self.focus.take() {
                                        if let Some(state) =
                                            self.panes.get_mut(main_window.id, window, pane)
                                        {
                                            state.buffer =
                                                Buffer::from(data::Buffer::Upstream(buffer));
                                            self.last_changed = Some(Instant::now());

                                            commands.extend(vec![
                                                self.reset_pane(main_window, window, pane),
                                                self.focus_pane(main_window, window, pane),
                                            ]);
                                        }
                                    }

                                    (Task::batch(commands), None)
                                }
                                command_bar::Buffer::Popout => {
                                    (self.popout_pane(main_window), None)
                                }
                                command_bar::Buffer::Merge => {
                                    (self.merge_pane(config, main_window), None)
                                }
                                command_bar::Buffer::ToggleInternal(buffer) => (
                                    self.toggle_internal_buffer(config, main_window, buffer),
                                    None,
                                ),
                            },
                            command_bar::Command::Configuration(command) => match command {
                                command_bar::Configuration::OpenDirectory => {
                                    let _ = open::that_detached(Config::config_dir());
                                    (Task::none(), None)
                                }
                                command_bar::Configuration::OpenWebsite => {
                                    let _ = open::that_detached(environment::WIKI_WEBSITE);
                                    (Task::none(), None)
                                }
                                command_bar::Configuration::Reload => {
                                    (Task::perform(Config::load(), Message::ConfigReloaded), None)
                                }
                            },
                            command_bar::Command::UI(command) => match command {
                                command_bar::Ui::ToggleSidebarVisibility => {
                                    self.side_menu.toggle_visibility();
                                    (Task::none(), None)
                                }
                            },
                            command_bar::Command::Theme(command) => match command {
                                command_bar::Theme::Switch(new) => {
                                    *theme = Theme::from(new);
                                    (Task::none(), None)
                                }
                                command_bar::Theme::OpenEditor => {
                                    if let Some(editor) = &self.theme_editor {
                                        (window::gain_focus(editor.window), None)
                                    } else {
                                        let (editor, task) = ThemeEditor::open(main_window);

                                        self.theme_editor = Some(editor);

                                        (task.then(|_| Task::none()), None)
                                    }
                                }
                            },
                        };

                        return (
                            Task::batch(vec![
                                command,
                                self.toggle_command_bar(
                                    &closed_buffers(self, main_window.id, clients),
                                    version,
                                    config,
                                    theme,
                                    main_window,
                                ),
                            ]),
                            event,
                        );
                    }
                    Some(command_bar::Event::Unfocused) => {
                        return (
                            self.toggle_command_bar(
                                &closed_buffers(self, main_window.id, clients),
                                version,
                                config,
                                theme,
                                main_window,
                            ),
                            None,
                        );
                    }
                    None => {}
                }
            }
            Message::Shortcut(shortcut) => {
                use shortcut::Command::*;

                // Only works on main window / pane_grid
                let mut move_focus = |direction: pane_grid::Direction| {
                    if let Some((window, pane)) = self.focus.as_ref() {
                        if *window == main_window.id {
                            if let Some(adjacent) = self.panes.main.adjacent(*pane, direction) {
                                return self.focus_pane(main_window, *window, adjacent);
                            }
                        }
                    } else if let Some((pane, _)) = self.panes.main.panes.iter().next() {
                        return self.focus_pane(main_window, main_window.id, *pane);
                    }

                    Task::none()
                };

                match shortcut {
                    MoveUp => return (move_focus(pane_grid::Direction::Up), None),
                    MoveDown => return (move_focus(pane_grid::Direction::Down), None),
                    MoveLeft => return (move_focus(pane_grid::Direction::Left), None),
                    MoveRight => return (move_focus(pane_grid::Direction::Right), None),
                    CloseBuffer => {
                        if let Some((window, pane)) = self.focus {
                            return (self.close_pane(main_window, window, pane), None);
                        }
                    }
                    MaximizeBuffer => {
                        if let Some((window, pane)) = self.focus.as_ref() {
                            // Only main window has >1 pane to maximize
                            if *window == main_window.id {
                                self.panes.main.maximize(*pane);
                            }
                        }
                    }
                    RestoreBuffer => {
                        self.panes.main.restore();
                    }
                    CycleNextBuffer => {
                        let all_buffers = all_buffers(clients, &self.history);
                        let open_buffers = open_buffers(self, main_window.id);

                        if let Some((window, pane, state)) = self.get_focused_mut(main_window) {
                            if let Some(buffer) = cycle_next_buffer(
                                state.buffer.upstream(),
                                all_buffers,
                                &open_buffers,
                            ) {
                                state.buffer = Buffer::from(data::Buffer::Upstream(buffer));
                                self.focus = None;
                                return (self.focus_pane(main_window, window, pane), None);
                            }
                        }
                    }
                    CyclePreviousBuffer => {
                        let all_buffers = all_buffers(clients, &self.history);
                        let open_buffers = open_buffers(self, main_window.id);

                        if let Some((window, pane, state)) = self.get_focused_mut(main_window) {
                            if let Some(buffer) = cycle_previous_buffer(
                                state.buffer.upstream(),
                                all_buffers,
                                &open_buffers,
                            ) {
                                state.buffer = Buffer::from(data::Buffer::Upstream(buffer));
                                self.focus = None;
                                return (self.focus_pane(main_window, window, pane), None);
                            }
                        }
                    }
                    LeaveBuffer => {
                        if let Some((_, _, state)) = self.get_focused_mut(main_window) {
                            if let Some(buffer) = state.buffer.upstream().cloned() {
                                return self.leave_buffer(main_window, clients, buffer);
                            }
                        }
                    }
                    ToggleNicklist => {
                        if let Some((_, _, pane)) = self.get_focused_mut(main_window) {
                            pane.update_settings(|settings| {
                                settings.channel.nicklist.enabled =
                                    !settings.channel.nicklist.enabled
                            });
                        }
                    }
                    ToggleTopic => {
                        if let Some((_, _, pane)) = self.get_focused_mut(main_window) {
                            pane.update_settings(|settings| {
                                settings.channel.topic.enabled = !settings.channel.topic.enabled
                            });
                        }
                    }
                    ToggleSidebar => {
                        self.side_menu.toggle_visibility();
                    }
                    CommandBar => {
                        return (
                            self.toggle_command_bar(
                                &closed_buffers(self, main_window.id, clients),
                                version,
                                config,
                                theme,
                                main_window,
                            ),
                            None,
                        );
                    }
                    ReloadConfiguration => {
                        return (Task::perform(Config::load(), Message::ConfigReloaded), None);
                    }
                    FileTransfers => {
                        return (
                            self.toggle_internal_buffer(
                                config,
                                main_window,
                                buffer::Internal::FileTransfers,
                            ),
                            None,
                        );
                    }
                    Logs => {
                        return (
                            self.toggle_internal_buffer(
                                config,
                                main_window,
                                buffer::Internal::Logs,
                            ),
                            None,
                        );
                    }
                    ThemeEditor => {
                        return (self.toggle_theme_editor(theme, main_window), None);
                    }
                    Highlight => {
                        return (
                            self.toggle_internal_buffer(
                                config,
                                main_window,
                                buffer::Internal::Highlights,
                            ),
                            None,
                        );
                    }
                }
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
        clients: &'a client::Map,
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
                        clients,
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
        clients: &'a client::Map,
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
            anchored_overlay(
                background,
                command_bar
                    .view(
                        &all_buffers(clients, &self.history),
                        self.focus,
                        self.buffer_resize_action(),
                        version,
                        config,
                        main_window.id,
                    )
                    .map(Message::Task),
                anchored_overlay::Anchor::BelowTopCentered,
                10.0,
            )
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
        clients: &data::client::Map,
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
                        &closed_buffers(self, main_window.id, clients),
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
