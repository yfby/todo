use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};
use std::{io, u8};

mod sample;
mod task;

const SAVE_FILE: &str = "tasks.json";

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))
}

struct App {
    exit: bool,
    current_layout: CurrentLayout,
    current_interface: CurrentInterface,
    task_collection: task::TaskListCollection,
}

enum CurrentLayout {
    Task,
    Exit,
}

enum CurrentInterface {
    TaskMenu,
    TaskBody,
    Write,
    Exit,
}

struct Write {
    item_index: u8,
    input: String,
    interface: CurrentInterface,
    character_index: u8,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            current_layout: CurrentLayout::Task,
            current_interface: CurrentInterface::TaskMenu,
            task_collection: task::load_or_default(SAVE_FILE),
        }
    }
}

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
}

/// Event Logic
impl App {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.current_interface {
            CurrentInterface::TaskMenu => self.key_event_task_menu(key_event),
            CurrentInterface::TaskBody => todo!(),
            CurrentInterface::Write => todo!(),
            CurrentInterface::Exit => todo!(),
        }
    }

    fn key_event_task_menu(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('a') => {
                todo!()
            }
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('w') => self.save(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn save(&self) {
        if let Err(error) = task::save_to_file(&self.task_collection, SAVE_FILE) {
            eprintln!("Problem opening the file: {:?}", error);
        }
    }
}

/// Rendering Logic
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.current_layout {
            CurrentLayout::Task => self.render_list(area, buf),
            CurrentLayout::Exit => todo!(),
        }
    }
}

impl App {
    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let list_menu_chunk = chunks[0];
        let list_body_chunk = chunks[1];

        let list_menu =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(list_menu_chunk);
        let list_body =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(list_body_chunk);

        let items: Vec<ListItem> = self
            .task_collection
            .lists()
            .iter()
            .map(|tl| ListItem::from(tl.name().to_string()))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Task Menu")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .highlight_symbol(">> ");

        let mut list_state = ListState::default().with_selected(Some(0));

        StatefulWidget::render(list, list_menu[1], buf, &mut list_state);
    }
}
