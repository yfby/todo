use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget},
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
    current_layout: CurrentLayout,
    current_interface: CurrentInterface,
    previous_layout: CurrentLayout,
    previous_interface: CurrentInterface,
    task_collection: task::TaskListCollection,
    menu_state: ListState,
    write_input: WriteInterface,
    cursor_position: Option<Position>,
}

#[derive(Clone, Copy)]
enum CurrentLayout {
    Task,
    Exit,
}

#[derive(PartialEq, Clone, Copy)]
enum CurrentInterface {
    TaskMenu,
    TaskBody,
    Write,
    Exit,
}

struct WriteInterface {
    input: String,
    character_index: usize,
}

impl WriteInterface {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.input.clear();
        self.character_index = 0;
    }

    fn final_input(&self) -> &str {
        &self.input
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            current_layout: CurrentLayout::Task,
            current_interface: CurrentInterface::TaskMenu,
            previous_layout: CurrentLayout::Task,
            previous_interface: CurrentInterface::TaskMenu,
            task_collection: task::load_or_default(SAVE_FILE),
            menu_state: ListState::default().with_selected(Some(0)),
            write_input: WriteInterface {
                input: String::new(),
                character_index: 0,
            },
            cursor_position: None,
        }
    }
}

/// Root Logic
impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());

                // set cursor position for writing
                if let Some(pos) = self.cursor_position {
                    frame.set_cursor_position(pos);
                }
            })?;

            self.handle_events()?;
        }
        Ok(())
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
            CurrentInterface::Write => self.key_event_write(key_event),
            CurrentInterface::Exit => todo!(),
        }
    }

    fn key_event_task_menu(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('k') | KeyCode::Down => self.menu_state.select_next(),
            KeyCode::Char('j') | KeyCode::Up => self.menu_state.select_previous(),
            KeyCode::Char('l') | KeyCode::Left | KeyCode::Enter => self.menu_state.select(None),
            KeyCode::Char('a') => {
                self.previous_layout = self.current_layout;
                self.previous_interface = self.current_interface;
                self.write_input.input = String::new();
                self.write_input.reset_cursor();

                self.current_interface = CurrentInterface::Write;
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(index) = self.menu_state.selected() {
                    self.task_collection.remove_list(index);
                }
            }
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('w') => self.save(),
            _ => {}
        }
    }

    fn key_event_write(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                self.task_collection
                    .add_list(task::TaskList::new(self.write_input.final_input()));

                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
            KeyCode::Char(to_insert) => self.write_input.enter_char(to_insert),
            KeyCode::Backspace => self.write_input.delete_char(),
            KeyCode::Left => self.write_input.move_cursor_left(),
            KeyCode::Right => self.write_input.move_cursor_right(),
            KeyCode::Esc => {
                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
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
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.current_layout {
            CurrentLayout::Task => self.render_task_layout(area, buf),
            CurrentLayout::Exit => todo!(),
        }
    }
}

impl App {
    fn render_task_layout(&mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let task_menu_area =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(chunks[0]);

        self.render_task_menu(task_menu_area[1], buf);

        // input widget for task
        if self.current_interface == CurrentInterface::Write {
            let write_block = Block::bordered()
                .title("Task Menu")
                .title_alignment(Alignment::Center);

            Paragraph::new(self.write_input.final_input())
                .block(write_block)
                .render(task_menu_area[0], buf);

            self.cursor_position = Some(Position::new(
                task_menu_area[0].x + self.write_input.character_index as u16 + 1,
                task_menu_area[0].y + 1,
            ));
        } else {
            self.cursor_position = None;
        }
    }

    fn render_task_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<_> = self
            .task_collection
            .lists()
            .iter()
            .map(|item| item.name())
            .collect();

        let block = Block::bordered()
            .title("Task Menu")
            .title_alignment(Alignment::Left);

        if items.is_empty() {
            Paragraph::new("No Tasks Found")
                .block(block)
                .centered()
                .render(area, buf);
        } else {
            let list = List::new(items)
                .block(block)
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
                .highlight_symbol(">> ");

            StatefulWidget::render(list, area, buf, &mut self.menu_state);
        }
    }
}
