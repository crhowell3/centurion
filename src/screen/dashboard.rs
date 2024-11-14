pub struct Dashboard {
    theme_editor: Option<ThemeEditor>,
}

#[derive(Debug)]
pub enum Message {
    ThemeEditor(theme_editor::Message),
}

#[derive(Debug)]
pub enum Event {
    ReloadThemes,
    Exit,
}

impl Dashboard {
    pub fn update(
        &mut self,
        message: Message,
        theme: &mut Theme,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::ThemeEditor(message) => {
                let mut editor_event = None;
                let mut event = None;
                let mut tasks = vec![];

                if let some(editor) = self.theme_editor.as_mut() {
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
        }
        (Task::none(), None)
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Message> {}
}
