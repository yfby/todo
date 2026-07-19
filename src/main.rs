use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::io;

mod sample;
mod task;

const SAVE_FILE: &str = "tasks.json";

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))
}

struct App {
    exit: bool,
    current_screen: CurrentScreen,
    task_collection: task::TaskListCollection,
}

pub enum CurrentScreen {
    ListMenu,
    ListBody,
    Exit,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            current_screen: CurrentScreen::ListMenu,
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
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(frame.area());
        frame.render_widget("H000O", layout[1]);
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
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

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        todo!()
    }
}
