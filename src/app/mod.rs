pub(crate) mod event;
pub(crate) mod ui;
mod write;

use color_eyre::Result;
use crossterm::event::{Event, KeyEventKind};
use ratatui::{DefaultTerminal, layout::Position, widgets::ListState};
use std::io;
use std::path::PathBuf;

use crate::task;
use write::{WriteInterface, WriteType};

pub struct App {
    pub(crate) exit: bool,
    pub(crate) current_layout: CurrentLayout,
    pub(crate) current_interface: CurrentInterface,
    pub(crate) previous_layout: CurrentLayout,
    pub(crate) previous_interface: CurrentInterface,
    pub(crate) save_path: Option<PathBuf>,
    pub(crate) original_task_collection: task::TaskListCollection,
    pub(crate) task_collection: task::TaskListCollection,
    pub(crate) menu_state: ListState,
    pub(crate) task_state: ListState,
    pub(crate) write_input: WriteInterface,
    pub(crate) cursor_position: Option<Position>,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CurrentLayout {
    Task,
    Help,
    #[allow(dead_code)]
    Exit,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum CurrentInterface {
    TaskMenu,
    TaskBody,
    Write,
    Help,
    #[allow(dead_code)]
    Exit, // TODO: confirm exit
}

impl Default for App {
    fn default() -> Self {
        let save_path = task::save_file_path();
        let load = |path: &Option<PathBuf>| -> task::TaskListCollection {
            path.as_ref()
                .and_then(|p| p.to_str())
                .map(task::load_or_default)
                .unwrap_or_default()
        };
        let original = load(&save_path);
        Self {
            exit: false,
            current_layout: CurrentLayout::Task,
            current_interface: CurrentInterface::TaskMenu,
            previous_layout: CurrentLayout::Task,
            previous_interface: CurrentInterface::TaskMenu,
            save_path,
            original_task_collection: original.clone(),
            task_collection: original,
            menu_state: ListState::default().with_selected(Some(0)),
            task_state: ListState::default().with_selected(None),
            write_input: WriteInterface {
                input: String::new(),
                character_index: 0,
                write_type: WriteType::Menu,
            },
            cursor_position: None,
            error_message: None,
        }
    }
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());

                if let Some(pos) = self.cursor_position {
                    frame.set_cursor_position(pos);
                }
            })?;

            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match crossterm::event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
}
